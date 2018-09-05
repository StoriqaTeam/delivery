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
use validator::Validate;

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
use services::companies::{CompaniesService, CompaniesServiceImpl};
use services::companies_packages::{CompaniesPackagesService, CompaniesPackagesServiceImpl};
use services::countries::{CountriesService, CountriesServiceImpl};
use services::packages::{PackagesService, PackagesServiceImpl};
use services::products::{ProductsService, ProductsServiceImpl};
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

        let countries_service = CountriesServiceImpl::new(self.db_pool.clone(), self.cpu_pool.clone(), user_id, self.repo_factory.clone());

        let products_service = ProductsServiceImpl::new(self.db_pool.clone(), self.cpu_pool.clone(), user_id, self.repo_factory.clone());

        let companies_service = CompaniesServiceImpl::new(self.db_pool.clone(), self.cpu_pool.clone(), user_id, self.repo_factory.clone());

        let packages_service = PackagesServiceImpl::new(self.db_pool.clone(), self.cpu_pool.clone(), user_id, self.repo_factory.clone());

        let companies_packages_service =
            CompaniesPackagesServiceImpl::new(self.db_pool.clone(), self.cpu_pool.clone(), user_id, self.repo_factory.clone());

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

            // POST /products/<base_product_id>
            (&Post, Some(Route::ProductsById { base_product_id })) => {
                debug!(
                    "User with id = '{:?}' is requesting  // POST /products/{}",
                    user_id, base_product_id
                );
                serialize_future(
                    parse_body::<NewShipping>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // POST /products in NewShipping failed!")
                                .context(Error::Parse)
                                .into()
                        })
                        .and_then(move |new_shipping| products_service.upsert(base_product_id, new_shipping)),
                )
            }

            // GET /products/<base_product_id>
            (&Get, Some(Route::ProductsById { base_product_id })) => {
                debug!("User with id = '{:?}' is requesting  // GET /products/{}", user_id, base_product_id);
                serialize_future(products_service.get_by_base_product_id(base_product_id))
            }

            // DELETE /products/<base_product_id>
            (&Delete, Some(Route::ProductsById { base_product_id })) => {
                debug!(
                    "User with id = '{:?}' is requesting  // DELETE /products/{}",
                    user_id, base_product_id
                );
                serialize_future(products_service.delete(base_product_id))
            }

            // PUT /products/<base_product_id>/company_package/<company_package_id>
            (
                &Put,
                Some(Route::ProductsByIdAndCompanyPackageId {
                    base_product_id,
                    company_package_id,
                }),
            ) => {
                debug!(
                    "User with id = '{:?}' is requesting  // PUT /products/{}/company_package/{}",
                    user_id, base_product_id, company_package_id
                );
                serialize_future(
                    parse_body::<UpdateProducts>(req.body())
                        .map_err(move |e| {
                            e.context(format!(
                                "Parsing body // PUT /products/{}/company_package/{} in UpdateProducts failed!",
                                base_product_id, company_package_id
                            )).context(Error::Parse)
                                .into()
                        })
                        .and_then(move |update_products| products_service.update(base_product_id, company_package_id, update_products)),
                )
            }

            // POST /companies
            (&Post, Some(Route::Companies)) => {
                debug!("User with id = '{:?}' is requesting  // POST /companies", user_id);
                serialize_future(
                    parse_body::<NewCompany>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // POST /companies in NewCompanies failed!")
                                .context(Error::Parse)
                                .into()
                        })
                        .and_then(move |new_company| companies_service.create(new_company)),
                )
            }

            // GET /companies/<company_id>
            (&Get, Some(Route::CompanyById { company_id })) => {
                debug!("User with id = '{:?}' is requesting  // GET /companies/{}", user_id, company_id);
                serialize_future(companies_service.find(company_id))
            }

            // PUT /companies/<company_id>
            (&Put, Some(Route::CompanyById { company_id })) => {
                debug!("User with id = '{:?}' is requesting  // PUT /companies/{}", user_id, company_id);
                serialize_future(
                    parse_body::<UpdateCompany>(req.body())
                        .map_err(move |e| {
                            e.context(format!("Parsing body // PUT /companies/{} in UpdateCompany failed!", company_id))
                                .context(Error::Parse)
                                .into()
                        })
                        .and_then(move |update_company| companies_service.update(company_id, update_company)),
                )
            }

            // DELETE /companies/<company_id>
            (&Delete, Some(Route::CompanyById { company_id })) => {
                debug!("User with id = '{:?}' is requesting  // DELETE /companies/{}", user_id, company_id);
                serialize_future(companies_service.delete(company_id))
            }

            // POST /companies_packages
            (&Post, Some(Route::CompaniesPackages)) => {
                debug!("User with id = '{:?}' is requesting  // POST /companies_packages", user_id);
                serialize_future(
                    parse_body::<NewCompaniesPackages>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // POST /companies in NewCompaniesPackages failed!")
                                .context(Error::Parse)
                                .into()
                        })
                        .and_then(move |new_companies_packages| companies_packages_service.create(new_companies_packages)),
                )
            }

            // GET /available_packages
            (&Get, Some(Route::AvailablePackages)) => {
                debug!("User with id = '{:?}' is requesting  // GET /available_packages", user_id);
                if let (Some(country), Some(size), Some(weight)) =
                    parse_query!(req.query().unwrap_or_default(), "country" => CountryLabel, "size" => f64, "weight" => f64)
                {
                    serialize_future(companies_packages_service.find_available_from(country, size, weight))
                } else {
                    Box::new(future::err(
                        format_err!("Parsing query parameters // GET /available_packages failed!")
                            .context(Error::Parse)
                            .into(),
                    ))
                }
            }

            // GET /available_packages_for_user/<base_product_id>
            (&Get, Some(Route::AvailablePackagesForUser { base_product_id })) => {
                debug!(
                    "User with id = '{:?}' is requesting  // GET /available_packages_for_user/{}",
                    user_id, base_product_id
                );
                if let Some(user_country) = parse_query!(req.query().unwrap_or_default(), "user_country" => CountryLabel) {
                    serialize_future(products_service.find_available_to(base_product_id, user_country))
                } else {
                    Box::new(future::err(
                        format_err!(
                            "Parsing query parameters // GET /available_packages_for_user/{} failed!",
                            base_product_id
                        ).context(Error::Parse)
                            .into(),
                    ))
                }
            }

            // Get /companies_packages/<company_package_id>
            (&Get, Some(Route::CompaniesPackagesById { company_package_id })) => {
                debug!(
                    "User with id = '{:?}' is requesting  // GET /companies_packages/{}",
                    user_id, company_package_id
                );
                serialize_future(companies_packages_service.get(company_package_id))
            }

            // DELETE /companies_packages/<company_package_id>
            (&Delete, Some(Route::CompaniesPackagesById { company_package_id })) => {
                debug!(
                    "User with id = '{:?}' is requesting  // DELETE /companies_packages/{}",
                    user_id, company_package_id
                );
                serialize_future(companies_packages_service.delete(company_package_id))
            }

            // GET /countries
            (&Get, Some(Route::Countries)) => {
                debug!("User with id = '{:?}' is requesting  // GET /countries", user_id);
                serialize_future(countries_service.get_all())
            }

            // POST /countries
            (&Post, Some(Route::Countries)) => {
                debug!("User with id = '{:?}' is requesting  // POST /countries", user_id);
                serialize_future(
                    parse_body::<NewCountry>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // POST /countries in NewCountry failed!")
                                .context(Error::Parse)
                                .into()
                        })
                        .and_then(move |new_country| {
                            new_country
                                .validate()
                                .map_err(|e| e.context("Validation of NewCountry failed!").context(Error::Parse).into())
                                .into_future()
                                .and_then(move |_| countries_service.create(new_country))
                        }),
                )
            }

            // POST /packages
            (&Post, Some(Route::Packages)) => {
                debug!("User with id = '{:?}' is requesting  // POST /packages", user_id);
                serialize_future(
                    parse_body::<NewPackages>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // POST /packages in NewPackages failed!")
                                .context(Error::Parse)
                                .into()
                        })
                        .and_then(move |new_package| packages_service.create(new_package)),
                )
            }

            // GET /packages/<package_id>
            (&Get, Some(Route::PackagesById { package_id })) => {
                debug!("User with id = '{:?}' is requesting  // GET /packages/{}", user_id, package_id);
                serialize_future(packages_service.find(package_id))
            }

            // GET /packages
            (&Get, Some(Route::Packages)) => {
                debug!("User with id = '{:?}' is requesting  // GET /packages", user_id);
                serialize_future(packages_service.list())
            }

            // PUT /packages/<package_id>
            (&Put, Some(Route::PackagesById { package_id })) => {
                debug!("User with id = '{:?}' is requesting  // PUT /packages/{}", user_id, package_id);
                serialize_future(
                    parse_body::<UpdatePackages>(req.body())
                        .map_err(move |e| {
                            e.context(format!("Parsing body // PUT /packages/{} in UpdatePackages failed!", package_id))
                                .context(Error::Parse)
                                .into()
                        })
                        .and_then(move |update_package| packages_service.update(package_id, update_package)),
                )
            }

            // DELETE /packages/<package_id>
            (&Delete, Some(Route::PackagesById { package_id })) => {
                debug!("User with id = '{:?}' is requesting  // DELETE /packages/{}", user_id, package_id);
                serialize_future(packages_service.delete(package_id))
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
