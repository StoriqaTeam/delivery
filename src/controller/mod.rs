pub mod context;
pub mod routes;

use std::str::FromStr;

use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Fail;
use futures::future;
use futures::prelude::*;
use hyper::header::Authorization;
use hyper::server::Request;
use hyper::{Delete, Get, Post, Put};
use r2d2::ManageConnection;
use validator::Validate;

use stq_http::controller::Controller;
use stq_http::controller::ControllerFuture;
use stq_http::errors::ErrorMessageWrapper;
use stq_http::request_util::parse_body;
use stq_http::request_util::serialize_future;
use stq_types::*;

use self::context::{DynamicContext, StaticContext};
use self::routes::Route;
use errors::Error;
use models::*;
use repos::repo_factory::*;
use repos::CountrySearch;
use sentry_integration::log_and_capture_error;
use services::companies::CompaniesService;
use services::companies_packages::CompaniesPackagesService;
use services::countries::CountriesService;
use services::packages::PackagesService;
use services::products::ProductsService;
use services::user_addresses::UserAddressService;
use services::user_roles::UserRolesService;
use services::Service;

/// Controller handles route parsing and calling `Service` layer
pub struct ControllerImpl<T, M, F>
where
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    M: ManageConnection<Connection = T>,
    F: ReposFactory<T>,
{
    pub static_context: StaticContext<T, M, F>,
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > ControllerImpl<T, M, F>
{
    /// Create a new controller based on services
    pub fn new(static_context: StaticContext<T, M, F>) -> Self {
        Self { static_context }
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

        let dynamic_context = DynamicContext::new(user_id);
        let service = Service::new(self.static_context.clone(), dynamic_context);

        let path = req.path().to_string();

        let fut = match (&req.method().clone(), self.static_context.route_parser.test(req.path())) {
            (Get, Some(Route::RolesByUserId { user_id })) => {
                debug!("Received request to get roles by user id {}", user_id);
                serialize_future({ service.get_roles(user_id) })
            }
            (Post, Some(Route::Roles)) => serialize_future({
                parse_body::<NewUserRole>(req.body()).and_then(move |data| {
                    debug!("Received request to create role {:?}", data);
                    service.create_role(data)
                })
            }),
            (Delete, Some(Route::RolesByUserId { user_id })) => {
                debug!("Received request to delete role by user id {}", user_id);
                serialize_future({ service.delete_by_user_id(user_id) })
            }
            (Delete, Some(Route::RoleById { id })) => {
                debug!("Received request to delete role by id {}", id);
                serialize_future({ service.delete_by_id(id) })
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
                        }).and_then(move |new_shipping| service.upsert(base_product_id, new_shipping)),
                )
            }

            // GET /products/<base_product_id>
            (&Get, Some(Route::ProductsById { base_product_id })) => {
                debug!("User with id = '{:?}' is requesting  // GET /products/{}", user_id, base_product_id);
                serialize_future(service.get_by_base_product_id(base_product_id))
            }

            // DELETE /products/<base_product_id>
            (&Delete, Some(Route::ProductsById { base_product_id })) => {
                debug!(
                    "User with id = '{:?}' is requesting  // DELETE /products/{}",
                    user_id, base_product_id
                );
                serialize_future(service.delete_products(base_product_id))
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
                        }).and_then(move |update_products| service.update_products(base_product_id, company_package_id, update_products)),
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
                        }).and_then(move |new_company| service.create_company(new_company)),
                )
            }

            // GET /companies
            (&Get, Some(Route::Companies)) => {
                debug!("User with id = '{:?}' is requesting  // GET /companies", user_id);
                serialize_future(service.list_companies())
            }

            // GET /companies/<company_id>
            (&Get, Some(Route::CompanyById { company_id })) => {
                debug!("User with id = '{:?}' is requesting  // GET /companies/{}", user_id, company_id);
                serialize_future(service.find_company(company_id))
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
                        }).and_then(move |update_company| service.update_company(company_id, update_company)),
                )
            }

            // DELETE /companies/<company_id>
            (&Delete, Some(Route::CompanyById { company_id })) => {
                debug!("User with id = '{:?}' is requesting  // DELETE /companies/{}", user_id, company_id);
                serialize_future(service.delete_company(company_id))
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
                        }).and_then(move |new_companies_packages| service.create_company_package(new_companies_packages)),
                )
            }

            // GET /available_packages
            (&Get, Some(Route::AvailablePackages)) => {
                debug!("User with id = '{:?}' is requesting  // GET /available_packages", user_id);
                if let (Some(country), Some(size), Some(weight)) =
                    parse_query!(req.query().unwrap_or_default(), "country" => Alpha3, "size" => f64, "weight" => f64)
                {
                    serialize_future(service.get_available_packages(country, size, weight))
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
                if let Some(user_country) = parse_query!(req.query().unwrap_or_default(), "user_country" => Alpha3) {
                    serialize_future(service.find_available_shipping_for_user(base_product_id, user_country))
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

            // GET /available_packages_for_user/products/:id/companies_packages/:id
            (
                &Get,
                Some(Route::AvailablePackageForUser {
                    base_product_id,
                    company_package_id,
                }),
            ) => {
                debug!(
                    "User with id = '{:?}' is requesting  // GET /available_packages_for_user/products/{}/companies_packages/{}",
                    user_id, base_product_id, company_package_id
                );
                serialize_future(service.get_available_package_for_user(base_product_id, company_package_id))
            }

            // Get /companies_packages/<company_package_id>
            (&Get, Some(Route::CompaniesPackagesById { company_package_id })) => {
                debug!(
                    "User with id = '{:?}' is requesting  // GET /companies_packages/{}",
                    user_id, company_package_id
                );
                serialize_future(service.get_company_package(company_package_id))
            }

            // Get /packages/<package_id>/companies
            (&Get, Some(Route::CompaniesByPackageId { package_id })) => {
                debug!(
                    "User with id = '{:?}' is requesting // GET /packages/{}/companies",
                    user_id, package_id
                );
                serialize_future(service.get_companies(package_id))
            }

            // Get /companies/<company_id>/packages
            (&Get, Some(Route::PackagesByCompanyId { company_id })) => {
                debug!(
                    "User with id = '{:?}' is requesting // GET /companies/{}/packages",
                    user_id, company_id
                );
                serialize_future(service.get_packages(company_id))
            }

            // DELETE /companies/<company_id>/packages/<package_id>
            (&Delete, Some(Route::CompaniesPackagesByIds { company_id, package_id })) => {
                debug!(
                    "User with id = '{:?}' is requesting  // DELETE /companies/{}/packages/{}",
                    user_id, company_id, package_id
                );
                serialize_future(service.delete_company_package(company_id, package_id))
            }

            // GET /countries
            (&Get, Some(Route::Countries)) => {
                debug!("User with id = '{:?}' is requesting  // GET /countries", user_id);
                serialize_future(service.get_all())
            }

            // GET /countries/flatten
            (&Get, Some(Route::CountriesFlatten)) => {
                debug!("User with id = '{:?}' is requesting  // GET /countries/flatten", user_id);
                serialize_future(service.get_all_flatten())
            }

            // Get /countries/alpha2/<alpha2>
            (&Get, Some(Route::CountryByAlpha2 { alpha2 })) => {
                debug!("User with id = '{:?}' is requesting  // GET /countries/alpha2/{}", user_id, alpha2);
                let search = CountrySearch::Alpha2(alpha2);
                serialize_future(service.find_country(search))
            }

            // Get /countries/alpha3/<alpha3>
            (&Get, Some(Route::CountryByAlpha3 { alpha3 })) => {
                debug!("User with id = '{:?}' is requesting  // GET /countries/alpha3/{}", user_id, alpha3);
                let search = CountrySearch::Alpha3(alpha3);
                serialize_future(service.find_country(search))
            }

            // Get /countries/numeric/<numeric_id>
            (&Get, Some(Route::CountryByNumeric { numeric })) => {
                debug!(
                    "User with id = '{:?}' is requesting  // GET /countries/numeric/{}",
                    user_id, numeric
                );
                let search = CountrySearch::Numeric(numeric);
                serialize_future(service.find_country(search))
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
                        }).and_then(move |new_country| {
                            new_country
                                .validate()
                                .map_err(|e| format_err!("Validation of NewCountry failed!").context(Error::Validate(e)).into())
                                .into_future()
                                .and_then(move |_| service.create_country(new_country))
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
                        }).and_then(move |new_package| service.create_package(new_package)),
                )
            }

            // GET /packages/<package_id>
            (&Get, Some(Route::PackagesById { package_id })) => {
                debug!("User with id = '{:?}' is requesting  // GET /packages/{}", user_id, package_id);
                serialize_future(service.find_packages(package_id))
            }

            // GET /packages
            (&Get, Some(Route::Packages)) => {
                debug!("User with id = '{:?}' is requesting  // GET /packages", user_id);
                serialize_future(service.list_packages())
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
                        }).and_then(move |update_package| service.update_package(package_id, update_package)),
                )
            }

            // DELETE /packages/<package_id>
            (&Delete, Some(Route::PackagesById { package_id })) => {
                debug!("User with id = '{:?}' is requesting  // DELETE /packages/{}", user_id, package_id);
                serialize_future(service.delete_package(package_id))
            }

            // GET /users/<user_id>/addresses
            (&Get, Some(Route::UserAddress { user_id })) => {
                debug!("Received request to get addresses for user {}", user_id);
                serialize_future(service.get_addresses(user_id))
            }

            // POST /users/addresses
            (&Post, Some(Route::UsersAddresses)) => {
                debug!("Received request to create delivery address with user id {:?}", user_id);
                serialize_future(
                    parse_body::<NewUserAddress>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // POST /users/addresses in NewUserAddress failed!")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |new_address| {
                            new_address
                                .validate()
                                .map_err(|e| {
                                    format_err!("Validation of NewUserAddress failed!")
                                        .context(Error::Validate(e))
                                        .into()
                                }).into_future()
                                .and_then(move |_| service.create_address(new_address))
                        }),
                )
            }

            // PUT /users/addresses/<id>
            (&Put, Some(Route::UserAddressById { user_address_id })) => {
                debug!("Received request to update user address {}", user_address_id);
                serialize_future(
                    parse_body::<UpdateUserAddress>(req.body())
                        .map_err(move |e| {
                            e.context(format!(
                                "Parsing body PUT /users/addresses/{} in UpdateUserAddress failed!",
                                user_address_id
                            )).context(Error::Parse)
                            .into()
                        }).and_then(move |new_address| {
                            new_address
                                .validate()
                                .map_err(|e| {
                                    format_err!("Validation of UpdateUserAddress failed!")
                                        .context(Error::Validate(e))
                                        .into()
                                }).into_future()
                                .and_then(move |_| service.update_address(user_address_id, new_address))
                        }),
                )
            }

            // DELETE /users/addresses/<id>
            (&Delete, Some(Route::UserAddressById { user_address_id })) => {
                debug!("Received request to delete user  address with id {}", user_address_id);
                serialize_future(service.delete_address(user_address_id))
            }

            // Fallback
            (m, _) => Box::new(future::err(
                format_err!("Request to non existing endpoint in delivery microservice! {:?} {:?}", m, path)
                    .context(Error::NotFound)
                    .into(),
            )),
        }.map_err(|err| {
            let wrapper = ErrorMessageWrapper::<Error>::from(&err);
            if wrapper.inner.code == 500 {
                log_and_capture_error(&err);
            }
            err
        });

        Box::new(fut)
    }
}
