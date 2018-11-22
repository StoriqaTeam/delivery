use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Error as FailureError;
use std::sync::Arc;
use stq_cache::cache::{Cache, CacheSingle};
use stq_types::*;

use models::*;
use repos::legacy_acl::{Acl, SystemACL};
use repos::*;

pub trait ReposFactory<C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static>: Clone + Send + 'static {
    fn create_companies_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<CompaniesRepo + 'a>;
    fn create_companies_packages_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<CompaniesPackagesRepo + 'a>;
    fn create_countries_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<CountriesRepo + 'a>;
    fn create_products_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<ProductsRepo + 'a>;
    fn create_packages_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<PackagesRepo + 'a>;
    fn create_pickups_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<PickupsRepo + 'a>;
    fn create_users_addresses_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<UserAddressesRepo + 'a>;
    fn create_user_roles_repo_with_sys_acl<'a>(&self, db_conn: &'a C) -> Box<UserRolesRepo + 'a>;
    fn create_user_roles_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<UserRolesRepo + 'a>;
}

pub struct ReposFactoryImpl<C1, C2>
where
    C1: CacheSingle<Country>,
    C2: Cache<Vec<DeliveryRole>>,
{
    country_cache: Arc<CountryCacheImpl<C1>>,
    roles_cache: Arc<RolesCacheImpl<C2>>,
}

impl<C1, C2> Clone for ReposFactoryImpl<C1, C2>
where
    C1: CacheSingle<Country>,
    C2: Cache<Vec<DeliveryRole>>,
{
    fn clone(&self) -> Self {
        Self {
            country_cache: self.country_cache.clone(),
            roles_cache: self.roles_cache.clone(),
        }
    }
}

impl<C1, C2> ReposFactoryImpl<C1, C2>
where
    C1: CacheSingle<Country> + Send + Sync + 'static,
    C2: Cache<Vec<DeliveryRole>> + Send + Sync + 'static,
{
    pub fn new(country_cache: CountryCacheImpl<C1>, roles_cache: RolesCacheImpl<C2>) -> Self {
        Self {
            country_cache: Arc::new(country_cache),
            roles_cache: Arc::new(roles_cache),
        }
    }

    pub fn get_roles<'a, C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static>(
        &self,
        id: UserId,
        db_conn: &'a C,
    ) -> Vec<DeliveryRole> {
        self.create_user_roles_repo_with_sys_acl(db_conn)
            .list_for_user(id)
            .ok()
            .unwrap_or_default()
    }

    fn get_acl<'a, T, C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static>(
        &self,
        db_conn: &'a C,
        user_id: Option<UserId>,
    ) -> Box<Acl<Resource, Action, Scope, FailureError, T>> {
        user_id.map_or(
            Box::new(UnauthorizedAcl::default()) as Box<Acl<Resource, Action, Scope, FailureError, T>>,
            |id| {
                let roles = self.get_roles(id, db_conn);
                (Box::new(ApplicationAcl::new(roles, id)) as Box<Acl<Resource, Action, Scope, FailureError, T>>)
            },
        )
    }
}

impl<C, C1, C2> ReposFactory<C> for ReposFactoryImpl<C1, C2>
where
    C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    C1: CacheSingle<Country> + Send + Sync + 'static,
    C2: Cache<Vec<DeliveryRole>> + Send + Sync + 'static,
{
    fn create_companies_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<CompaniesRepo + 'a> {
        let acl = self.get_acl(db_conn, user_id);
        let all_countries = self.create_countries_repo(db_conn, user_id).get_all().ok().unwrap_or_default();
        Box::new(CompaniesRepoImpl::new(db_conn, acl, all_countries)) as Box<CompaniesRepo>
    }

    fn create_companies_packages_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<CompaniesPackagesRepo + 'a> {
        let acl = self.get_acl(db_conn, user_id);
        let all_countries = self.create_countries_repo(db_conn, user_id).get_all().ok().unwrap_or_default();
        Box::new(CompaniesPackagesRepoImpl::new(db_conn, acl, all_countries)) as Box<CompaniesPackagesRepo>
    }

    fn create_countries_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<CountriesRepo + 'a> {
        let acl = self.get_acl(db_conn, user_id);
        let cache = self.country_cache.clone();
        Box::new(CountriesRepoImpl::new(db_conn, acl, cache)) as Box<CountriesRepo>
    }

    fn create_products_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<ProductsRepo + 'a> {
        let acl = self.get_acl(db_conn, user_id);
        let all_countries = self.create_countries_repo(db_conn, user_id).get_all().ok().unwrap_or_default();
        Box::new(ProductsRepoImpl::new(db_conn, acl, all_countries)) as Box<ProductsRepo>
    }

    fn create_packages_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<PackagesRepo + 'a> {
        let acl = self.get_acl(db_conn, user_id);
        let all_countries = self.create_countries_repo(db_conn, user_id).get_all().ok().unwrap_or_default();
        Box::new(PackagesRepoImpl::new(db_conn, acl, all_countries)) as Box<PackagesRepo>
    }

    fn create_pickups_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<PickupsRepo + 'a> {
        let acl = self.get_acl(db_conn, user_id);
        Box::new(PickupsRepoImpl::new(db_conn, acl)) as Box<PickupsRepo>
    }

    fn create_users_addresses_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<UserAddressesRepo + 'a> {
        let acl = self.get_acl(db_conn, user_id);
        Box::new(UserAddressesRepoImpl::new(db_conn, acl)) as Box<UserAddressesRepo>
    }

    fn create_user_roles_repo_with_sys_acl<'a>(&self, db_conn: &'a C) -> Box<UserRolesRepo + 'a> {
        let cache = self.roles_cache.clone();
        Box::new(UserRolesRepoImpl::new(
            db_conn,
            Box::new(SystemACL::default()) as Box<Acl<Resource, Action, Scope, FailureError, UserRole>>,
            cache,
        )) as Box<UserRolesRepo>
    }
    fn create_user_roles_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<UserRolesRepo + 'a> {
        let acl = self.get_acl(db_conn, user_id);
        let cache = self.roles_cache.clone();
        Box::new(UserRolesRepoImpl::new(db_conn, acl, cache)) as Box<UserRolesRepo>
    }
}

#[cfg(test)]
pub mod tests {

    extern crate r2d2;
    extern crate stq_http;

    use std::error::Error;
    use std::fmt;
    use std::sync::Arc;
    use std::time::SystemTime;

    use diesel::connection::AnsiTransactionManager;
    use diesel::connection::SimpleConnection;
    use diesel::deserialize::QueryableByName;
    use diesel::pg::Pg;
    use diesel::query_builder::AsQuery;
    use diesel::query_builder::QueryFragment;
    use diesel::query_builder::QueryId;
    use diesel::sql_types::HasSqlType;
    use diesel::Connection;
    use diesel::ConnectionResult;
    use diesel::QueryResult;
    use diesel::Queryable;
    use futures_cpupool::CpuPool;
    use r2d2::ManageConnection;
    use tokio_core::reactor::Handle;

    use stq_static_resources::Currency;
    use stq_types::*;

    use config::Config;
    use controller::context::{DynamicContext, StaticContext};
    use models::*;
    use repos::*;
    use services::*;

    pub const MOCK_REPO_FACTORY: ReposFactoryMock = ReposFactoryMock {};
    pub static MOCK_USER_ID: UserId = UserId(1);
    pub static MOCK_STORE_ID: StoreId = StoreId(1);
    pub static MOCK_BASE_PRODUCT_ID: BaseProductId = BaseProductId(1);

    #[derive(Default, Copy, Clone)]
    pub struct ReposFactoryMock;

    impl<C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> ReposFactory<C> for ReposFactoryMock {
        fn create_companies_repo<'a>(&self, _db_conn: &'a C, _user_id: Option<UserId>) -> Box<CompaniesRepo + 'a> {
            Box::new(CompaniesRepoMock::default()) as Box<CompaniesRepo>
        }

        fn create_companies_packages_repo<'a>(&self, _db_conn: &'a C, _user_id: Option<UserId>) -> Box<CompaniesPackagesRepo + 'a> {
            Box::new(CompaniesPackagesRepoMock::default()) as Box<CompaniesPackagesRepo>
        }

        fn create_countries_repo<'a>(&self, _db_conn: &'a C, _user_id: Option<UserId>) -> Box<CountriesRepo + 'a> {
            Box::new(CountriesRepoMock::default()) as Box<CountriesRepo>
        }

        fn create_products_repo<'a>(&self, _db_conn: &'a C, _user_id: Option<UserId>) -> Box<ProductsRepo + 'a> {
            Box::new(ProductsRepoMock::default()) as Box<ProductsRepo>
        }

        fn create_packages_repo<'a>(&self, _db_conn: &'a C, _user_id: Option<UserId>) -> Box<PackagesRepo + 'a> {
            Box::new(PackagesRepoMock::default()) as Box<PackagesRepo>
        }

        fn create_pickups_repo<'a>(&self, _db_conn: &'a C, _user_id: Option<UserId>) -> Box<PickupsRepo + 'a> {
            Box::new(PickupsRepoMock::default()) as Box<PickupsRepo>
        }

        fn create_users_addresses_repo<'a>(&self, _db_conn: &'a C, _user_id: Option<UserId>) -> Box<UserAddressesRepo + 'a> {
            Box::new(UserAddressesRepoMock::default()) as Box<UserAddressesRepo>
        }

        fn create_user_roles_repo<'a>(&self, _db_conn: &'a C, _user_id: Option<UserId>) -> Box<UserRolesRepo + 'a> {
            Box::new(UserRolesRepoMock::default()) as Box<UserRolesRepo>
        }
        fn create_user_roles_repo_with_sys_acl<'a>(&self, _db_conn: &'a C) -> Box<UserRolesRepo + 'a> {
            Box::new(UserRolesRepoMock::default()) as Box<UserRolesRepo>
        }
    }

    pub fn create_service(
        user_id: Option<UserId>,
        handle: Arc<Handle>,
    ) -> Service<MockConnection, MockConnectionManager, ReposFactoryMock> {
        let manager = MockConnectionManager::default();
        let db_pool = r2d2::Pool::builder().build(manager).expect("Failed to create connection pool");
        let cpu_pool = CpuPool::new(1);

        let config = Config::new().unwrap();
        let client = stq_http::client::Client::new(&config.to_http_config(), &handle);
        let client_handle = client.handle();
        let static_context = StaticContext::new(db_pool, cpu_pool, client_handle, Arc::new(config), MOCK_REPO_FACTORY);
        let dynamic_context = DynamicContext::new(user_id, String::default());

        Service::new(static_context, dynamic_context)
    }

    #[derive(Clone, Default)]
    pub struct UserRolesRepoMock;

    impl UserRolesRepo for UserRolesRepoMock {
        fn list_for_user(&self, user_id_value: UserId) -> RepoResult<Vec<DeliveryRole>> {
            Ok(match user_id_value.0 {
                1 => vec![DeliveryRole::Superuser],
                _ => vec![DeliveryRole::User],
            })
        }

        fn create(&self, payload: NewUserRole) -> RepoResult<UserRole> {
            Ok(UserRole {
                id: RoleId::new(),
                user_id: payload.user_id,
                name: payload.name,
                data: None,
            })
        }

        fn delete_by_user_id(&self, user_id_arg: UserId) -> RepoResult<Vec<UserRole>> {
            Ok(vec![UserRole {
                id: RoleId::new(),
                user_id: user_id_arg,
                name: DeliveryRole::User,
                data: None,
            }])
        }

        fn delete_by_id(&self, id: RoleId) -> RepoResult<UserRole> {
            Ok(UserRole {
                id,
                user_id: UserId(1),
                name: DeliveryRole::User,
                data: None,
            })
        }
    }

    #[derive(Clone, Default)]
    pub struct ProductsRepoMock;

    impl ProductsRepo for ProductsRepoMock {
        /// Create a new products
        fn create(&self, payload: NewProducts) -> RepoResult<Products> {
            Ok(Products {
                id: ShippingId(1),
                base_product_id: payload.base_product_id,
                store_id: payload.store_id,
                company_package_id: payload.company_package_id,
                shipping: payload.shipping,
                price: payload.price,
                deliveries_to: payload.deliveries_to,
            })
        }

        /// Create many a new products
        fn create_many(&self, payloads: Vec<NewProducts>) -> RepoResult<Vec<Products>> {
            let mut result = vec![];
            for item in payloads {
                result.push(Products {
                    id: ShippingId(1),
                    base_product_id: item.base_product_id,
                    store_id: item.store_id,
                    company_package_id: item.company_package_id,
                    shipping: item.shipping,
                    price: item.price,
                    deliveries_to: item.deliveries_to,
                });
            }

            Ok(result)
        }

        /// Get a products
        fn get_by_base_product_id(&self, base_product_id: BaseProductId) -> RepoResult<Vec<Products>> {
            Ok(vec![Products {
                id: ShippingId(1),
                base_product_id: base_product_id,
                store_id: StoreId(1),
                company_package_id: CompanyPackageId(1),
                shipping: ShippingVariant::Local,
                price: None,
                deliveries_to: vec![],
            }])
        }

        fn get_products_countries(&self, base_product_id: BaseProductId) -> RepoResult<Vec<ProductsWithAvailableCountries>> {
            let product = Products {
                id: ShippingId(1),
                base_product_id,
                store_id: StoreId(1),
                company_package_id: CompanyPackageId(1),
                shipping: ShippingVariant::Local,
                price: None,
                deliveries_to: vec![],
            };

            Ok(vec![ProductsWithAvailableCountries(product, vec![])])
        }

        /// find available product delivery to users country
        fn find_available_to(&self, _base_product_id: BaseProductId, _user_country: Alpha3) -> RepoResult<Vec<AvailablePackageForUser>> {
            Ok(vec![AvailablePackageForUser {
                id: CompanyPackageId(1),
                shipping_id: ShippingId(1),
                shipping_variant: ShippingVariant::Local,
                name: "UPS-avia".to_string(),
                logo: "logo".to_string(),
                price: None,
                store_id: MOCK_STORE_ID,
                base_product_id: MOCK_BASE_PRODUCT_ID,
                deliveries_to: vec![],
            }])
        }

        fn get_available_package_for_user(
            &self,
            _base_product_id_arg: BaseProductId,
            _package_id_arg: CompanyPackageId,
        ) -> RepoResult<Option<AvailablePackageForUser>> {
            Ok(None)
        }

        fn get_available_package_for_user_by_shipping_id(&self, _shipping_id: ShippingId) -> RepoResult<Option<AvailablePackageForUser>> {
            Ok(None)
        }

        /// Update a products
        fn update(
            &self,
            base_product_id_arg: BaseProductId,
            company_package_id: CompanyPackageId,
            payload: UpdateProducts,
        ) -> RepoResult<Products> {
            Ok(Products {
                id: ShippingId(1),
                base_product_id: base_product_id_arg,
                store_id: StoreId(1),
                company_package_id,
                shipping: payload.shipping.unwrap(),
                price: payload.price,
                deliveries_to: payload.deliveries_to.unwrap_or_default(),
            })
        }

        /// Delete a products
        fn delete(&self, base_product_id_arg: BaseProductId) -> RepoResult<Vec<Products>> {
            Ok(vec![Products {
                id: ShippingId(1),
                base_product_id: base_product_id_arg,
                store_id: StoreId(1),
                company_package_id: CompanyPackageId(1),
                shipping: ShippingVariant::Local,
                price: None,
                deliveries_to: vec![],
            }])
        }
    }

    #[derive(Clone, Default)]
    pub struct CountriesRepoMock;

    impl CountriesRepo for CountriesRepoMock {
        /// Find specific country by label
        fn find(&self, arg: Alpha3) -> RepoResult<Option<Country>> {
            Ok(Some(Country {
                label: CountryLabel("Russia".to_string()),
                children: vec![],
                level: 2,
                parent: Some("XEU".to_string().into()),
                alpha2: Alpha2("RU".to_string()),
                alpha3: arg,
                numeric: 0,
                is_selected: false,
            }))
        }

        fn find_by(&self, search: CountrySearch) -> RepoResult<Option<Country>> {
            match search {
                CountrySearch::Label(label) => Ok(Some(Country {
                    label,
                    children: vec![],
                    level: 2,
                    parent: Some("XEU".to_string().into()),
                    alpha2: Alpha2("RU".to_string()),
                    alpha3: Alpha3("RUS".to_string()),
                    numeric: 0,
                    is_selected: false,
                })),
                CountrySearch::Alpha2(alpha2) => Ok(Some(Country {
                    label: CountryLabel("Russia".to_string()),
                    children: vec![],
                    level: 2,
                    parent: Some("XEU".to_string().into()),
                    alpha2,
                    alpha3: Alpha3("RUS".to_string()),
                    numeric: 0,
                    is_selected: false,
                })),
                CountrySearch::Alpha3(alpha3) => Ok(Some(Country {
                    label: CountryLabel("Russia".to_string()),
                    children: vec![],
                    level: 2,
                    parent: Some("XEU".to_string().into()),
                    alpha2: Alpha2("RU".to_string()),
                    alpha3,
                    numeric: 0,
                    is_selected: false,
                })),
                CountrySearch::Numeric(numeric) => Ok(Some(Country {
                    label: CountryLabel("Russia".to_string()),
                    children: vec![],
                    level: 2,
                    parent: Some("XEU".to_string().into()),
                    alpha2: Alpha2("RU".to_string()),
                    alpha3: Alpha3("RUS".to_string()),
                    numeric,
                    is_selected: false,
                })),
            }
        }

        /// Creates new country
        fn create(&self, payload: NewCountry) -> RepoResult<Country> {
            Ok(Country {
                label: payload.label,
                children: vec![],
                level: payload.level,
                parent: None,
                alpha2: Alpha2("RU".to_string()),
                alpha3: Alpha3("RUS".to_string()),
                numeric: 0,
                is_selected: false,
            })
        }

        /// Returns all countries as a tree
        fn get_all(&self) -> RepoResult<Country> {
            Ok(create_mock_countries())
        }

        /// Returns all countries as a flatten vec
        fn get_all_flatten(&self) -> RepoResult<Vec<Country>> {
            Ok(create_mock_countries_flatten())
        }
    }

    fn create_mock_countries() -> Country {
        let country_3 = Country {
            label: "RUS".to_string().into(),
            children: vec![],
            level: 2,
            parent: Some("XEU".to_string().into()),
            alpha2: Alpha2("RU".to_string()),
            alpha3: Alpha3("RUS".to_string()),
            numeric: 0,
            is_selected: false,
        };
        let country_2 = Country {
            label: "Russia".to_string().into(),
            children: vec![country_3],
            level: 1,
            parent: Some("XEU".to_string().into()),
            alpha2: Alpha2("RU".to_string()),
            alpha3: Alpha3("RUS".to_string()),
            numeric: 0,
            is_selected: false,
        };
        Country {
            label: "Russia".to_string().into(),
            level: 2,
            parent: None,
            children: vec![country_2],
            alpha2: Alpha2("RU".to_string()),
            alpha3: Alpha3("RUS".to_string()),
            numeric: 0,
            is_selected: false,
        }
    }

    fn create_mock_countries_flatten() -> Vec<Country> {
        vec![Country {
            label: "RUS".to_string().into(),
            children: vec![],
            level: 2,
            parent: Some("XEU".to_string().into()),
            alpha2: Alpha2("RU".to_string()),
            alpha3: Alpha3("RUS".to_string()),
            numeric: 0,
            is_selected: false,
        }]
    }

    #[derive(Clone, Default)]
    pub struct CompaniesRepoMock;

    impl CompaniesRepo for CompaniesRepoMock {
        fn create(&self, payload: NewCompany) -> RepoResult<Company> {
            let payload = payload.to_raw()?;

            let raw = CompanyRaw {
                id: CompanyId(1),
                name: payload.name,
                label: payload.label,
                description: payload.description,
                deliveries_from: payload.deliveries_from,
                logo: payload.logo,
                currency: payload.currency,
            };

            let countries_arg = create_mock_countries();

            Ok(Company::from_raw(raw, &countries_arg)?)
        }

        fn list(&self) -> RepoResult<Vec<Company>> {
            Ok(vec![
                Company {
                    id: CompanyId(1),
                    name: "UPS Russia".to_string(),
                    label: "UPS".to_string(),
                    description: None,
                    deliveries_from: vec![],
                    logo: "".to_string(),
                    currency: Currency::STQ,
                },
                Company {
                    id: CompanyId(2),
                    name: "UPS USA".to_string(),
                    label: "UPS".to_string(),
                    description: None,
                    deliveries_from: vec![],
                    logo: "".to_string(),
                    currency: Currency::USD,
                },
            ])
        }

        fn find(&self, _company_id: CompanyId) -> RepoResult<Option<Company>> {
            Ok(None)
        }

        fn find_deliveries_from(&self, _country: Alpha3) -> RepoResult<Vec<Company>> {
            Ok(vec![
                Company {
                    id: CompanyId(1),
                    name: "UPS Russia".to_string(),
                    label: "UPS".to_string(),
                    description: None,
                    deliveries_from: vec![],
                    logo: "".to_string(),
                    currency: Currency::STQ,
                },
                Company {
                    id: CompanyId(2),
                    name: "UPS USA".to_string(),
                    label: "UPS".to_string(),
                    description: None,
                    deliveries_from: vec![],
                    logo: "".to_string(),
                    currency: Currency::USD,
                },
            ])
        }

        fn update(&self, id_arg: CompanyId, payload: UpdateCompany) -> RepoResult<Company> {
            Ok(Company {
                id: id_arg,
                name: payload.name.unwrap(),
                label: payload.label.unwrap(),
                description: payload.description,
                deliveries_from: vec![],
                logo: payload.logo.unwrap(),
                currency: payload.currency.unwrap(),
            })
        }

        fn delete(&self, id_arg: CompanyId) -> RepoResult<Company> {
            Ok(Company {
                id: id_arg,
                name: "UPS USA".to_string(),
                label: "UPS".to_string(),
                description: None,
                deliveries_from: vec![],
                logo: "".to_string(),
                currency: Currency::STQ,
            })
        }
    }

    #[derive(Clone, Default)]
    pub struct PickupsRepoMock;

    impl PickupsRepo for PickupsRepoMock {
        fn create(&self, payload: NewPickups) -> RepoResult<Pickups> {
            Ok(Pickups {
                id: 1,
                base_product_id: payload.base_product_id,
                store_id: payload.store_id,
                pickup: payload.pickup,
                price: payload.price,
            })
        }

        fn list(&self) -> RepoResult<Vec<Pickups>> {
            Ok(vec![Pickups {
                id: 1,
                base_product_id: BaseProductId(1),
                store_id: StoreId(1),
                pickup: false,
                price: Some(ProductPrice(1.0)),
            }])
        }

        fn get(&self, base_product_id_arg: BaseProductId) -> RepoResult<Option<Pickups>> {
            Ok(Some(Pickups {
                id: 1,
                base_product_id: base_product_id_arg,
                store_id: StoreId(1),
                pickup: false,
                price: Some(ProductPrice(1.0)),
            }))
        }

        fn update(&self, base_product_id_arg: BaseProductId, payload: UpdatePickups) -> RepoResult<Pickups> {
            Ok(Pickups {
                id: 1,
                base_product_id: base_product_id_arg,
                store_id: StoreId(1),
                pickup: payload.pickup.unwrap(),
                price: payload.price,
            })
        }

        fn delete(&self, base_product_id_arg: BaseProductId) -> RepoResult<Option<Pickups>> {
            Ok(Some(Pickups {
                id: 1,
                base_product_id: base_product_id_arg,
                store_id: StoreId(1),
                pickup: false,
                price: Some(ProductPrice(1.0)),
            }))
        }
    }

    #[derive(Clone, Default)]
    pub struct PackagesRepoMock;

    impl PackagesRepo for PackagesRepoMock {
        fn create(&self, payload: NewPackages) -> RepoResult<Packages> {
            let payload = payload.to_raw()?;

            let raw = PackagesRaw {
                id: PackageId(1),
                name: payload.name,
                max_size: payload.max_size,
                min_size: payload.min_size,
                max_weight: payload.max_weight,
                min_weight: payload.min_weight,
                deliveries_to: payload.deliveries_to,
            };

            let countries_arg = create_mock_countries();

            Ok(raw.to_packages(&countries_arg)?)
        }

        fn find_deliveries_to(&self, _countries: Vec<Alpha3>) -> RepoResult<Vec<Packages>> {
            Ok(vec![Packages {
                id: PackageId(1),
                name: "package1".to_string(),
                max_size: 0f64,
                min_size: 0f64,
                max_weight: 0f64,
                min_weight: 0f64,
                deliveries_to: vec![],
            }])
        }

        fn list(&self) -> RepoResult<Vec<Packages>> {
            Ok(vec![Packages {
                id: PackageId(1),
                name: "package1".to_string(),
                max_size: 0f64,
                min_size: 0f64,
                max_weight: 0f64,
                min_weight: 0f64,
                deliveries_to: vec![],
            }])
        }

        fn find(&self, id_arg: PackageId) -> RepoResult<Option<Packages>> {
            Ok(Some(Packages {
                id: id_arg,
                name: "package1".to_string(),
                max_size: 0f64,
                min_size: 0f64,
                max_weight: 0f64,
                min_weight: 0f64,
                deliveries_to: vec![],
            }))
        }

        fn update(&self, id_arg: PackageId, payload: UpdatePackages) -> RepoResult<Packages> {
            Ok(Packages {
                id: id_arg,
                name: payload.name.unwrap(),
                max_size: payload.max_size.unwrap(),
                min_size: payload.min_size.unwrap(),
                max_weight: payload.max_weight.unwrap(),
                min_weight: payload.min_weight.unwrap(),
                deliveries_to: vec![],
            })
        }

        fn delete(&self, id_arg: PackageId) -> RepoResult<Packages> {
            Ok(Packages {
                id: id_arg,
                name: "package1".to_string(),
                max_size: 0f64,
                min_size: 0f64,
                max_weight: 0f64,
                min_weight: 0f64,
                deliveries_to: vec![],
            })
        }
    }

    #[derive(Clone, Default)]
    pub struct CompaniesPackagesRepoMock;

    impl CompaniesPackagesRepo for CompaniesPackagesRepoMock {
        /// Create a new companies_packages
        fn create(&self, payload: NewCompanyPackage) -> RepoResult<CompanyPackage> {
            let NewCompanyPackage {
                company_id,
                package_id,
                shipping_rate_source,
            } = payload;

            let shipping_rate_source = shipping_rate_source.unwrap_or_default();
            Ok(CompanyPackage {
                id: CompanyPackageId(1),
                company_id,
                package_id,
                shipping_rate_source,
            })
        }

        /// Getting available packages satisfying the constraints
        fn get_available_packages(
            &self,
            company_id_args: Vec<CompanyId>,
            _size: f64,
            _weight: f64,
            _deliveries_from: Alpha3,
        ) -> RepoResult<Vec<AvailablePackages>> {
            Ok(company_id_args
                .into_iter()
                .map(|id| AvailablePackages {
                    id: CompanyPackageId(id.0),
                    name: "name".to_string(),
                    logo: "logo".to_string(),
                    deliveries_to: vec![],
                    local_available: false,
                    currency: Currency::STQ,
                }).collect())
        }

        fn get(&self, id_arg: CompanyPackageId) -> RepoResult<Option<CompanyPackage>> {
            Ok(Some(CompanyPackage {
                id: id_arg,
                company_id: CompanyId(1),
                package_id: PackageId(1),
                shipping_rate_source: ShippingRateSource::NotAvailable,
            }))
        }

        /// Returns companies by package id
        fn get_companies(&self, _package_id: PackageId) -> RepoResult<Vec<Company>> {
            Ok(vec![Company {
                id: CompanyId(1),
                name: "UPS USA".to_string(),
                label: "UPS".to_string(),
                description: None,
                deliveries_from: vec![],
                currency: Currency::STQ,
                logo: "".to_string(),
            }])
        }

        /// Returns packages by company id
        fn get_packages(&self, _company_id: CompanyId) -> RepoResult<Vec<Packages>> {
            Ok(vec![Packages {
                id: PackageId(1),
                name: "package1".to_string(),
                max_size: 0f64,
                min_size: 0f64,
                max_weight: 0f64,
                min_weight: 0f64,
                deliveries_to: vec![],
            }])
        }

        /// Delete a companies_packages
        fn delete(&self, company_id_arg: CompanyId, package_id_arg: PackageId) -> RepoResult<CompanyPackage> {
            Ok(CompanyPackage {
                id: CompanyPackageId(1),
                company_id: company_id_arg,
                package_id: package_id_arg,
                shipping_rate_source: ShippingRateSource::NotAvailable,
            })
        }
    }

    #[derive(Clone, Default)]
    pub struct UserAddressesRepoMock;

    impl UserAddressesRepo for UserAddressesRepoMock {
        /// Returns list of user_delivery_address for a specific user
        fn list_for_user(&self, user_id: UserId) -> RepoResult<Vec<UserAddress>> {
            Ok(vec![UserAddress {
                id: 1,
                user_id,
                administrative_area_level_1: None,
                administrative_area_level_2: None,
                country: "None".to_string(),
                locality: None,
                political: None,
                postal_code: "None".to_string(),
                route: None,
                street_number: None,
                is_priority: true,
                address: None,
                created_at: SystemTime::now(),
                updated_at: SystemTime::now(),
                country_code: None,
            }])
        }

        /// Create a new user delivery address
        fn create(&self, payload: NewUserAddress) -> RepoResult<UserAddress> {
            Ok(UserAddress {
                id: 1,
                user_id: payload.user_id,
                administrative_area_level_1: payload.administrative_area_level_1,
                administrative_area_level_2: payload.administrative_area_level_2,
                country: payload.country,
                locality: payload.locality,
                political: payload.political,
                postal_code: payload.postal_code,
                route: payload.route,
                street_number: payload.street_number,
                is_priority: payload.is_priority,
                address: None,
                created_at: SystemTime::now(),
                updated_at: SystemTime::now(),
                country_code: payload.country_code,
            })
        }

        /// Update a user delivery address
        fn update(&self, id: i32, payload: UpdateUserAddress) -> RepoResult<UserAddress> {
            Ok(UserAddress {
                id,
                user_id: UserId(1),
                administrative_area_level_1: payload.administrative_area_level_1,
                administrative_area_level_2: payload.administrative_area_level_2,
                country: payload.country.unwrap_or_default(),
                locality: payload.locality,
                political: payload.political,
                postal_code: payload.postal_code.unwrap_or_default(),
                route: payload.route,
                street_number: payload.street_number,
                is_priority: payload.is_priority.unwrap_or_default(),
                address: None,
                created_at: SystemTime::now(),
                updated_at: SystemTime::now(),
                country_code: payload.country_code,
            })
        }

        /// Delete user delivery address
        fn delete(&self, id: i32) -> RepoResult<UserAddress> {
            Ok(UserAddress {
                id,
                user_id: UserId(1),
                administrative_area_level_1: None,
                administrative_area_level_2: None,
                country: "None".to_string(),
                locality: None,
                political: None,
                postal_code: "None".to_string(),
                route: None,
                street_number: None,
                is_priority: true,
                address: None,
                created_at: SystemTime::now(),
                updated_at: SystemTime::now(),
                country_code: None,
            })
        }
    }

    #[derive(Default)]
    pub struct MockConnection {
        tr: AnsiTransactionManager,
    }

    impl Connection for MockConnection {
        type Backend = Pg;
        type TransactionManager = AnsiTransactionManager;

        fn establish(_database_url: &str) -> ConnectionResult<MockConnection> {
            Ok(MockConnection::default())
        }

        fn execute(&self, _query: &str) -> QueryResult<usize> {
            unimplemented!()
        }

        fn query_by_index<T, U>(&self, _source: T) -> QueryResult<Vec<U>>
        where
            T: AsQuery,
            T::Query: QueryFragment<Pg> + QueryId,
            Pg: HasSqlType<T::SqlType>,
            U: Queryable<T::SqlType, Pg>,
        {
            unimplemented!()
        }

        fn query_by_name<T, U>(&self, _source: &T) -> QueryResult<Vec<U>>
        where
            T: QueryFragment<Pg> + QueryId,
            U: QueryableByName<Pg>,
        {
            unimplemented!()
        }

        fn execute_returning_count<T>(&self, _source: &T) -> QueryResult<usize>
        where
            T: QueryFragment<Pg> + QueryId,
        {
            unimplemented!()
        }

        fn transaction_manager(&self) -> &Self::TransactionManager {
            &self.tr
        }
    }

    impl SimpleConnection for MockConnection {
        fn batch_execute(&self, _query: &str) -> QueryResult<()> {
            Ok(())
        }
    }

    #[derive(Default)]
    pub struct MockConnectionManager;

    impl ManageConnection for MockConnectionManager {
        type Connection = MockConnection;
        type Error = MockError;

        fn connect(&self) -> Result<MockConnection, MockError> {
            Ok(MockConnection::default())
        }

        fn is_valid(&self, _conn: &mut MockConnection) -> Result<(), MockError> {
            Ok(())
        }

        fn has_broken(&self, _conn: &mut MockConnection) -> bool {
            false
        }
    }

    #[derive(Debug)]
    pub struct MockError {}

    impl fmt::Display for MockError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "SuperError is here!")
        }
    }

    impl Error for MockError {
        fn description(&self) -> &str {
            "I'm the superhero of errors"
        }

        fn cause(&self) -> Option<&Error> {
            None
        }
    }
}
