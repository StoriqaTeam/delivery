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
use stq_types::*;

use self::routes::Route;
use config;
use errors::Error;
use models::*;
use repos::acl::RolesCacheImpl;
use repos::repo_factory::*;
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

            // Fallback
            (m, _) => Box::new(future::err(
                format_err!("Request to non existing endpoint in delivery microservice! {:?} {:?}", m, path)
                    .context(Error::NotFound)
                    .into(),
            )),
        }
    }
}
