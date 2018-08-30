use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Error as FailureError;

use stq_types::*;

use models::*;
use repos::legacy_acl::{Acl, SystemACL};
use repos::*;

pub trait ReposFactory<C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static>: Clone + Send + 'static {
    fn create_user_roles_repo<'a>(&self, db_conn: &'a C) -> Box<UserRolesRepo + 'a>;
    fn create_companies_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<CompaniesRepo + 'a>;
    fn create_countries_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<CountriesRepo + 'a>;
    fn create_products_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<ProductsRepo + 'a>;
}

#[derive(Clone)]
pub struct ReposFactoryImpl {
    roles_cache: RolesCacheImpl,
    country_cache: CountryCacheImpl,
}

impl ReposFactoryImpl {
    pub fn new(roles_cache: RolesCacheImpl, country_cache: CountryCacheImpl) -> Self {
        Self {
            roles_cache,
            country_cache,
        }
    }

    pub fn get_roles<'a, C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static>(
        &self,
        id: UserId,
        db_conn: &'a C,
    ) -> Vec<StoresRole> {
        self.create_user_roles_repo(db_conn).list_for_user(id).ok().unwrap_or_default()
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

impl<C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> ReposFactory<C> for ReposFactoryImpl {
    fn create_user_roles_repo<'a>(&self, db_conn: &'a C) -> Box<UserRolesRepo + 'a> {
        Box::new(UserRolesRepoImpl::new(
            db_conn,
            Box::new(SystemACL::default()) as Box<Acl<Resource, Action, Scope, FailureError, UserRole>>,
            self.roles_cache.clone(),
        )) as Box<UserRolesRepo>
    }

    fn create_companies_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<CompaniesRepo + 'a> {
        let acl = self.get_acl(db_conn, user_id);
        Box::new(CompaniesRepoImpl::new(db_conn, acl)) as Box<CompaniesRepo>
    }

    fn create_countries_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<CountriesRepo + 'a> {
        let acl = self.get_acl(db_conn, user_id);
        Box::new(CountriesRepoImpl::new(db_conn, acl, self.country_cache.clone())) as Box<CountriesRepo>
    }

    fn create_products_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<ProductsRepo + 'a> {
        let acl = self.get_acl(db_conn, user_id);
        Box::new(ProductsRepoImpl::new(db_conn, acl)) as Box<ProductsRepo>
    }
}

#[cfg(test)]
pub mod tests {

    use std::error::Error;
    use std::fmt;

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
    use r2d2::ManageConnection;

    use stq_types::*;

    use models::*;
    use repos::*;

    pub const MOCK_REPO_FACTORY: ReposFactoryMock = ReposFactoryMock {};
    pub static MOCK_USER_ID: UserId = UserId(1);

    #[derive(Default, Copy, Clone)]
    pub struct ReposFactoryMock;

    impl<C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> ReposFactory<C> for ReposFactoryMock {
        fn create_user_roles_repo<'a>(&self, _db_conn: &'a C) -> Box<UserRolesRepo + 'a> {
            Box::new(UserRolesRepoMock::default()) as Box<UserRolesRepo>
        }

        fn create_companies_repo<'a>(&self, _db_conn: &'a C, _user_id: Option<UserId>) -> Box<CompaniesRepo + 'a> {
            Box::new(CompaniesRepoMock::default()) as Box<CompaniesRepo>
        }

        fn create_countries_repo<'a>(&self, _db_conn: &'a C, _user_id: Option<UserId>) -> Box<CountriesRepo + 'a> {
            Box::new(CountriesRepoMock::default()) as Box<CountriesRepo>
        }

        fn create_products_repo<'a>(&self, _db_conn: &'a C, _user_id: Option<UserId>) -> Box<ProductsRepo + 'a> {
            Box::new(ProductsRepoMock::default()) as Box<ProductsRepo>
        }
    }

    #[derive(Clone, Default)]
    pub struct UserRolesRepoMock;

    impl UserRolesRepo for UserRolesRepoMock {
        fn list_for_user(&self, user_id_value: UserId) -> RepoResult<Vec<StoresRole>> {
            Ok(match user_id_value.0 {
                1 => vec![StoresRole::Superuser],
                _ => vec![StoresRole::User],
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
                name: StoresRole::User,
                data: None,
            }])
        }

        fn delete_by_id(&self, id: RoleId) -> RepoResult<UserRole> {
            Ok(UserRole {
                id: id,
                user_id: UserId(1),
                name: StoresRole::User,
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
                id: 1,
                base_product_id: payload.base_product_id,
                store_id: payload.store_id,
                company_package_id: payload.company_package_id,
                price: payload.price,
                deliveries_to: payload.deliveries_to,
            })
        }

        /// Get a products
        fn get_by_base_product_id(&self, base_product_id: BaseProductId) -> RepoResult<Vec<Products>> {
            Ok(vec![Products {
                id: 1,
                base_product_id: base_product_id,
                store_id: StoreId(1),
                company_package_id: CompanyPackageId(1),
                price: None,
                deliveries_to: vec![],
            }])
        }

        /// Update a products
        fn update(
            &self,
            base_product_id_arg: BaseProductId,
            company_package_id: CompanyPackageId,
            payload: UpdateProducts,
        ) -> RepoResult<Products> {
            Ok(Products {
                id: 1,
                base_product_id: base_product_id_arg,
                store_id: StoreId(1),
                company_package_id: company_package_id,
                price: payload.price,
                deliveries_to: payload.deliveries_to.unwrap_or_default(),
            })
        }

        /// Delete a products
        fn delete(&self, base_product_id_arg: BaseProductId) -> RepoResult<Products> {
            Ok(Products {
                id: 1,
                base_product_id: base_product_id_arg,
                store_id: StoreId(1),
                company_package_id: CompanyPackageId(1),
                price: None,
                deliveries_to: vec![],
            })
        }
    }

    #[derive(Clone, Default)]
    pub struct CountriesRepoMock;

    impl CountriesRepo for CountriesRepoMock {
        /// Find specific country by label
        fn find(&self, label_arg: CountryLabel) -> RepoResult<Option<Country>> {
            Ok(Some(Country {
                label: label_arg,
                name: vec![],
                children: vec![],
                level: 3,
                parent_label: Some("EEE".to_string().into()),
            }))
        }

        /// Creates new country
        fn create(&self, payload: NewCountry) -> RepoResult<Country> {
            Ok(Country {
                label: payload.label,
                name: vec![],
                children: vec![],
                level: payload.level,
                parent_label: None,
            })
        }

        /// Returns all countries as a tree
        fn get_all(&self) -> RepoResult<Country> {
            Ok(create_mock_countries())
        }
    }

    fn create_mock_countries() -> Country {
        let country_3 = Country {
            label: "rus".to_string().into(),
            name: vec![],
            children: vec![],
            level: 3,
            parent_label: Some("EEE".to_string().into()),
        };
        let country_2 = Country {
            label: "EEE".to_string().into(),
            name: vec![],
            children: vec![country_3],
            level: 2,
            parent_label: Some("ALL".to_string().into()),
        };
        let country_1 = Country {
            label: "ALL".to_string().into(),
            name: vec![],
            children: vec![country_2],
            level: 1,
            parent_label: Some("root".to_string().into()),
        };
        Country {
            children: vec![country_1],
            ..Default::default()
        }
    }

    #[derive(Clone, Default)]
    pub struct CompaniesRepoMock;

    impl CompaniesRepo for CompaniesRepoMock {
        fn create(&self, payload: NewCompany) -> RepoResult<Company> {
            Ok(Company {
                id: CompanyId(1),
                name: payload.name,
                label: payload.label,
                description: payload.description,
                deliveries_from: payload.deliveries_from,
                logo: payload.logo,
            })
        }

        fn list(&self) -> RepoResult<Vec<Company>> {
            Ok(vec![
                Company {
                    id: CompanyId(1),
                    name: "UPS Russia".to_string(),
                    label: "UPS".to_string(),
                    description: None,
                    deliveries_from: DeliveriesFrom { country_labels: vec![] },
                    logo: "".to_string(),
                },
                Company {
                    id: CompanyId(2),
                    name: "UPS USA".to_string(),
                    label: "UPS".to_string(),
                    description: None,
                    deliveries_from: DeliveriesFrom { country_labels: vec![] },
                    logo: "".to_string(),
                },
            ])
        }

        fn find(&self, _company_id: CompanyId) -> RepoResult<Option<Company>> {
            Ok(None)
        }

        fn find_deliveries_from(&self, country: CountryLabel) -> RepoResult<Vec<Company>> {
            Ok(vec![
                Company {
                    id: CompanyId(1),
                    name: "UPS Russia".to_string(),
                    label: "UPS".to_string(),
                    description: None,
                    deliveries_from: DeliveriesFrom {
                        country_labels: vec![country.clone()],
                    },
                    logo: "".to_string(),
                },
                Company {
                    id: CompanyId(2),
                    name: "UPS USA".to_string(),
                    label: "UPS".to_string(),
                    description: None,
                    deliveries_from: DeliveriesFrom {
                        country_labels: vec![country.clone()],
                    },
                    logo: "".to_string(),
                },
            ])
        }

        fn update(&self, id_arg: CompanyId, payload: UpdateCompany) -> RepoResult<Company> {
            Ok(Company {
                id: id_arg,
                name: payload.name.unwrap(),
                label: payload.label.unwrap(),
                description: payload.description,
                deliveries_from: payload.deliveries_from.unwrap(),
                logo: payload.logo.unwrap(),
            })
        }

        fn delete(&self, id_arg: CompanyId) -> RepoResult<Company> {
            Ok(Company {
                id: id_arg,
                name: "UPS USA".to_string(),
                label: "UPS".to_string(),
                description: None,
                deliveries_from: DeliveriesFrom { country_labels: vec![] },
                logo: "".to_string(),
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
