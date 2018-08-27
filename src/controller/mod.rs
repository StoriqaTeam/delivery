pub mod routes;

use std::str::FromStr;
use std::sync::Arc;

use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Fail;
use futures::future;
use futures::prelude::*;
use futures_cpupool::CpuPool;
use hyper::header::Authorization;
use hyper::server::Request;
use hyper::{Delete, Get, Post, Put};
use r2d2::{ManageConnection, Pool};

use stq_http::client::ClientHandle;
use stq_http::controller::Controller;
use stq_http::controller::ControllerFuture;
use stq_http::request_util::parse_body;
use stq_http::request_util::serialize_future;
use stq_router::RouteParser;
use stq_static_resources::DeliveryCompany;
use stq_types::*;

use self::routes::Route;
use config;
use errors::Error;
use models::*;
use repos::acl::RolesCacheImpl;
use repos::repo_factory::*;
use services::delivery_to::{DeliveryToService, DeliveryToServiceImpl};
use services::international::{InternationalShippingService, InternationalShippingServiceImpl};
use services::local::{LocalShippingService, LocalShippingServiceImpl};
use services::restrictions::{RestrictionService, RestrictionServiceImpl};
use services::user_roles::{UserRolesService, UserRolesServiceImpl};

/// Controller handles route parsing and calling `Service` layer
#[derive(Clone)]
pub struct ControllerImpl<T, M, F>
where
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    M: ManageConnection<Connection = T>,
    F: ReposFactory<T>,
{
    pub db_pool: Pool<M>,
    pub config: config::Config,
    pub cpu_pool: CpuPool,
    pub route_parser: Arc<RouteParser<Route>>,
    pub repo_factory: F,
    pub roles_cache: RolesCacheImpl,
    pub http_client: ClientHandle,
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > ControllerImpl<T, M, F>
{
    /// Create a new controller based on services
    pub fn new(
        db_pool: Pool<M>,
        config: config::Config,
        cpu_pool: CpuPool,
        http_client: ClientHandle,
        roles_cache: RolesCacheImpl,
        repo_factory: F,
    ) -> Self {
        let route_parser = Arc::new(routes::create_route_parser());
        Self {
            db_pool,
            config,
            cpu_pool,
            route_parser,
            repo_factory,
            http_client,
            roles_cache,
        }
    }
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > Controller for ControllerImpl<T, M, F>
{
    /// Handle a request and get future response
    fn call(&self, req: Request) -> ControllerFuture {
        let headers = req.headers().clone();
        let auth_header = headers.get::<Authorization<String>>();
        let user_id = auth_header
            .map(|auth| auth.0.clone())
            .and_then(|id| i32::from_str(&id).ok())
            .map(UserId);

        debug!("User with id = '{:?}' is requesting {}", user_id, req.path());

        let cached_roles = self.roles_cache.clone();

        let user_roles_service =
            UserRolesServiceImpl::new(self.db_pool.clone(), self.cpu_pool.clone(), cached_roles, self.repo_factory.clone());
        let restrictions_service =
            RestrictionServiceImpl::new(self.db_pool.clone(), self.cpu_pool.clone(), user_id, self.repo_factory.clone());
        let local_shipping_service =
            LocalShippingServiceImpl::new(self.db_pool.clone(), self.cpu_pool.clone(), user_id, self.repo_factory.clone());
        let international_shipping_service =
            InternationalShippingServiceImpl::new(self.db_pool.clone(), self.cpu_pool.clone(), user_id, self.repo_factory.clone());

        let delivery_to_service =
            DeliveryToServiceImpl::new(self.db_pool.clone(), self.cpu_pool.clone(), user_id, self.repo_factory.clone());

        let path = req.path().to_string();

        match (&req.method().clone(), self.route_parser.test(req.path())) {
            (Get, Some(Route::RolesByUserId { user_id })) => {
                debug!("Received request to get roles by user id {}", user_id);
                serialize_future({ user_roles_service.get_roles(user_id) })
            }
            (Post, Some(Route::Roles)) => serialize_future({
                parse_body::<NewUserRole>(req.body()).and_then(move |data| {
                    debug!("Received request to create role {:?}", data);
                    user_roles_service.create(data)
                })
            }),
            (Delete, Some(Route::RolesByUserId { user_id })) => {
                debug!("Received request to delete role by user id {}", user_id);
                serialize_future({ user_roles_service.delete_by_user_id(user_id) })
            }
            (Delete, Some(Route::RoleById { id })) => {
                debug!("Received request to delete role by id {}", id);
                serialize_future({ user_roles_service.delete_by_id(id) })
            }

            // POST /restrictions
            (&Post, Some(Route::Restrictions)) => {
                debug!("User with id = '{:?}' is requesting  // POST /restrictions", user_id);
                serialize_future(
                    parse_body::<NewRestriction>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // POST /restrictions in NewRestriction failed!")
                                .context(Error::Parse)
                                .into()
                        })
                        .and_then(move |new_restriction| restrictions_service.create(new_restriction)),
                )
            }

            // GET /restrictions/<name>
            (&Get, Some(Route::Restrictions)) => {
                debug!("User with id = '{:?}' is requesting  // GET /restrictions", user_id);
                if let Some(name) = parse_query!(req.query().unwrap_or_default(), "name" => String) {
                    serialize_future(restrictions_service.get_by_name(name))
                } else {
                    Box::new(future::err(
                        format_err!("Parsing query parameters // GET /restrictions failed!")
                            .context(Error::Parse)
                            .into(),
                    ))
                }
            }

            // DELETE /restrictions/<name>
            (&Delete, Some(Route::Restrictions)) => {
                debug!("User with id = '{:?}' is requesting  // DELETE /restrictions", user_id);
                if let Some(name) = parse_query!(req.query().unwrap_or_default(), "name" => String) {
                    serialize_future(restrictions_service.delete(name))
                } else {
                    Box::new(future::err(
                        format_err!("Parsing query parameters // DELETE /restrictions failed!")
                            .context(Error::Parse)
                            .into(),
                    ))
                }
            }

            // PUT /restrictions
            (&Put, Some(Route::Restrictions)) => {
                debug!("User with id = '{:?}' is requesting  // PUT /restrictions", user_id);
                serialize_future(
                    parse_body::<UpdateRestriction>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // PUT /restrictions in UpdateRestriction failed!")
                                .context(Error::Parse)
                                .into()
                        })
                        .and_then(move |update_restriction| restrictions_service.update(update_restriction)),
                )
            }

            // POST /shipping/local
            (&Post, Some(Route::ShippingLocal)) => {
                debug!("User with id = '{:?}' is requesting  // POST /shipping/local", user_id);
                serialize_future(
                    parse_body::<NewLocalShipping>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // POST /shipping/local in NewLocalShipping failed!")
                                .context(Error::Parse)
                                .into()
                        })
                        .and_then(move |new_local_shipping| local_shipping_service.create(new_local_shipping)),
                )
            }

            // GET /shipping/local/<base_product_id>
            (&Get, Some(Route::ShippingLocalById { base_product_id })) => {
                debug!(
                    "User with id = '{:?}' is requesting  // GET /shipping/local/{}",
                    user_id, base_product_id
                );
                serialize_future(local_shipping_service.get_by_base_product_id(base_product_id))
            }

            // DELETE /shipping/local/<base_product_id>
            (&Delete, Some(Route::ShippingLocalById { base_product_id })) => {
                debug!(
                    "User with id = '{:?}' is requesting  // DELETE /shipping/local/{}",
                    user_id, base_product_id
                );
                serialize_future(local_shipping_service.delete(base_product_id))
            }

            // PUT /shipping/local/<base_product_id>
            (&Put, Some(Route::ShippingLocalById { base_product_id })) => {
                debug!(
                    "User with id = '{:?}' is requesting  // PUT /shipping/local/{}",
                    user_id, base_product_id
                );
                serialize_future(
                    parse_body::<UpdateLocalShipping>(req.body())
                        .map_err(move |e| {
                            e.context(format!(
                                "Parsing body // PUT /shipping/local/{} in UpdateLocalShipping failed!",
                                base_product_id
                            )).context(Error::Parse)
                                .into()
                        })
                        .and_then(move |update_local_shipping| local_shipping_service.update(base_product_id, update_local_shipping)),
                )
            }

            // POST /shipping/international
            (&Post, Some(Route::ShippingInternational)) => {
                debug!("User with id = '{:?}' is requesting  // POST /shipping/international", user_id);
                serialize_future(
                    parse_body::<NewInternationalShipping>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // POST /shipping/international in NewInternationalShipping failed!")
                                .context(Error::Parse)
                                .into()
                        })
                        .and_then(move |new_international_shipping| international_shipping_service.create(new_international_shipping)),
                )
            }

            // GET /shipping/international/<base_product_id>
            (&Get, Some(Route::ShippingInternationalById { base_product_id })) => {
                debug!(
                    "User with id = '{:?}' is requesting  // GET /shipping/international/{}",
                    user_id, base_product_id
                );
                serialize_future(international_shipping_service.get_by_base_product_id(base_product_id))
            }

            // DELETE /shipping/international/<base_product_id>
            (&Delete, Some(Route::ShippingInternationalById { base_product_id })) => {
                debug!(
                    "User with id = '{:?}' is requesting  // DELETE /shipping/international/{}",
                    user_id, base_product_id
                );
                serialize_future(international_shipping_service.delete(base_product_id))
            }

            // PUT /shipping/international/<base_product_id>
            (&Put, Some(Route::ShippingInternationalById { base_product_id })) => {
                debug!(
                    "User with id = '{:?}' is requesting  // PUT /shipping/international/{}",
                    user_id, base_product_id
                );
                serialize_future(
                    parse_body::<UpdateInternationalShipping>(req.body())
                        .map_err(move |e| {
                            e.context(format!(
                                "Parsing body // PUT /shipping/international/{} in UpdateInternationalShipping failed!",
                                base_product_id
                            )).context(Error::Parse)
                                .into()
                        })
                        .and_then(move |update_international_shipping| {
                            international_shipping_service.update(base_product_id, update_international_shipping)
                        }),
                )
            }
            // POST /delivery_to
            (&Post, Some(Route::DeliveryTo)) => {
                debug!("User with id = '{:?}' is requesting  // POST /delivery_to", user_id);
                serialize_future(
                    parse_body::<NewDeliveryTo>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // POST /delivery_to in NewDeliveryTo failed!")
                                .context(Error::Parse)
                                .into()
                        })
                        .and_then(move |new_delivery| delivery_to_service.create(new_delivery)),
                )
            }

            // GET /delivery_to/search/filters/company
            (&Get, Some(Route::DeliveryToFiltersCompany)) => {
                debug!(
                    "User with id = '{:?}' is requesting  // GET /delivery_to/search/filters/company",
                    user_id
                );
                if let Some(company_id) = parse_query!(req.query().unwrap_or_default(), "company_id" => DeliveryCompany) {
                    serialize_future(delivery_to_service.list_by_company(company_id))
                } else {
                    Box::new(future::err(
                        format_err!("Parsing query parameters // GET /delivery_to/search/filters/company failed!")
                            .context(Error::Parse)
                            .into(),
                    ))
                }
            }

            // GET /delivery_to/search/filters/country
            (&Get, Some(Route::DeliveryToFiltersCountry)) => {
                debug!(
                    "User with id = '{:?}' is requesting  // GET /delivery_to/search/filters/country",
                    user_id
                );
                if let Some(country) = parse_query!(req.query().unwrap_or_default(), "country" => String) {
                    serialize_future(delivery_to_service.list_by_country(country))
                } else {
                    Box::new(future::err(
                        format_err!("Parsing query parameters // GET /delivery_to/search/filters/country failed!")
                            .context(Error::Parse)
                            .into(),
                    ))
                }
            }

            // PUT /delivery_to
            (&Put, Some(Route::DeliveryTo)) => {
                debug!("User with id = '{:?}' is requesting  // PUT /delivery_to", user_id);
                serialize_future(
                    parse_body::<UpdateDeliveryTo>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // PUT /delivery_to in UpdateDeliveryTo failed!")
                                .context(Error::Parse)
                                .into()
                        })
                        .and_then(move |update_delivery| delivery_to_service.update(update_delivery)),
                )
            }

            // DELETE /delivery_to
            (&Delete, Some(Route::DeliveryTo)) => {
                debug!("User with id = '{:?}' is requesting  // DELETE /delivery_to", user_id);
                if let (Some(company_id), Some(country)) =
                    parse_query!(req.query().unwrap_or_default(), "company_id" => DeliveryCompany, "country" => String)
                {
                    serialize_future(delivery_to_service.delete(company_id, country))
                } else {
                    Box::new(future::err(
                        format_err!("Parsing query parameters // DELETE /delivery_to failed!")
                            .context(Error::Parse)
                            .into(),
                    ))
                }
            }

            // Fallback
            (m, _) => Box::new(future::err(
                format_err!("Request to non existing endpoint in delivery microservice! {:?} {:?}", m, path)
                    .context(Error::NotFound)
                    .into(),
            )),
        }
    }
}
