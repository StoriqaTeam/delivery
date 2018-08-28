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
    fn create_restrictions_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<RestrictionsRepo + 'a>;
    fn create_local_shippings_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<LocalShippingRepo + 'a>;
    fn create_international_shippings_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<InternationalShippingRepo + 'a>;
    fn create_delivery_to_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<DeliveryToRepo + 'a>;
    fn create_delivery_from_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<DeliveryFromRepo + 'a>;
    fn create_countries_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<CountriesRepo + 'a>;
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

    fn create_restrictions_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<RestrictionsRepo + 'a> {
        let acl = self.get_acl(db_conn, user_id);
        Box::new(RestrictionsRepoImpl::new(db_conn, acl)) as Box<RestrictionsRepo>
    }

    fn create_local_shippings_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<LocalShippingRepo + 'a> {
        let acl = self.get_acl(db_conn, user_id);
        Box::new(LocalShippingRepoImpl::new(db_conn, acl)) as Box<LocalShippingRepo>
    }

    fn create_international_shippings_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<InternationalShippingRepo + 'a> {
        let acl = self.get_acl(db_conn, user_id);
        Box::new(InternationalShippingRepoImpl::new(db_conn, acl)) as Box<InternationalShippingRepo>
    }

    fn create_delivery_to_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<DeliveryToRepo + 'a> {
        let acl = self.get_acl(db_conn, user_id);
        Box::new(DeliveryToRepoImpl::new(db_conn, acl)) as Box<DeliveryToRepo>
    }

    fn create_delivery_from_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<DeliveryFromRepo + 'a> {
        let acl = self.get_acl(db_conn, user_id);
        Box::new(DeliveryFromRepoImpl::new(db_conn, acl)) as Box<DeliveryFromRepo>
    }

    fn create_countries_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<CountriesRepo + 'a> {
        let acl = self.get_acl(db_conn, user_id);
        Box::new(CountriesRepoImpl::new(db_conn, acl, self.country_cache.clone())) as Box<CountriesRepo>
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
    use stq_static_resources::DeliveryCompany;

    pub const MOCK_REPO_FACTORY: ReposFactoryMock = ReposFactoryMock {};
    pub static MOCK_USER_ID: UserId = UserId(1);

    #[derive(Default, Copy, Clone)]
    pub struct ReposFactoryMock;

    impl<C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> ReposFactory<C> for ReposFactoryMock {
        fn create_user_roles_repo<'a>(&self, _db_conn: &'a C) -> Box<UserRolesRepo + 'a> {
            Box::new(UserRolesRepoMock::default()) as Box<UserRolesRepo>
        }

        fn create_restrictions_repo<'a>(&self, _db_conn: &'a C, _user_id: Option<UserId>) -> Box<RestrictionsRepo + 'a> {
            Box::new(RestrictionsRepoMock::default()) as Box<RestrictionsRepo>
        }

        fn create_local_shippings_repo<'a>(&self, _db_conn: &'a C, _user_id: Option<UserId>) -> Box<LocalShippingRepo + 'a> {
            Box::new(LocalShippingRepoMock::default()) as Box<LocalShippingRepo>
        }

        fn create_international_shippings_repo<'a>(
            &self,
            _db_conn: &'a C,
            _user_id: Option<UserId>,
        ) -> Box<InternationalShippingRepo + 'a> {
            Box::new(InternationalShippingRepoMock::default()) as Box<InternationalShippingRepo>
        }

        fn create_delivery_to_repo<'a>(&self, _db_conn: &'a C, _user_id: Option<UserId>) -> Box<DeliveryToRepo + 'a> {
            Box::new(DeliveryToRepoMock::default()) as Box<DeliveryToRepo>
        }

        fn create_delivery_from_repo<'a>(&self, _db_conn: &'a C, _user_id: Option<UserId>) -> Box<DeliveryFromRepo + 'a> {
            Box::new(DeliveryFromRepoMock::default()) as Box<DeliveryFromRepo>
        }

        fn create_countries_repo<'a>(&self, _db_conn: &'a C, _user_id: Option<UserId>) -> Box<CountriesRepo + 'a> {
            Box::new(CountriesRepoMock::default()) as Box<CountriesRepo>
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
    pub struct InternationalShippingRepoMock;

    impl InternationalShippingRepo for InternationalShippingRepoMock {
        /// Create a new local_shipping
        fn create(&self, payload: NewInternationalShipping) -> RepoResult<InternationalShipping> {
            Ok(InternationalShipping {
                id: 1,
                base_product_id: payload.base_product_id,
                store_id: payload.store_id,
                companies: payload.companies,
            })
        }

        /// Get a local_shipping
        fn get_by_base_product_id(&self, base_product_id: BaseProductId) -> RepoResult<InternationalShipping> {
            Ok(InternationalShipping {
                id: 1,
                base_product_id: base_product_id,
                store_id: StoreId(1),
                companies: vec![],
            })
        }

        /// Update a local_shipping
        fn update(&self, base_product_id_arg: BaseProductId, payload: UpdateInternationalShipping) -> RepoResult<InternationalShipping> {
            Ok(InternationalShipping {
                id: 1,
                base_product_id: base_product_id_arg,
                store_id: StoreId(1),
                companies: payload.companies.unwrap_or_default(),
            })
        }

        /// Delete a local_shipping
        fn delete(&self, base_product_id_arg: BaseProductId) -> RepoResult<InternationalShipping> {
            Ok(InternationalShipping {
                id: 1,
                base_product_id: base_product_id_arg,
                store_id: StoreId(1),
                companies: vec![],
            })
        }
    }

    #[derive(Clone, Default)]
    pub struct LocalShippingRepoMock;

    impl LocalShippingRepo for LocalShippingRepoMock {
        /// Create a new local_shipping
        fn create(&self, payload: NewLocalShipping) -> RepoResult<LocalShipping> {
            Ok(LocalShipping {
                id: 1,
                base_product_id: payload.base_product_id,
                store_id: payload.store_id,
                pickup: payload.pickup,
                pickup_price: payload.pickup_price,
                companies: payload.companies,
            })
        }

        /// Get a local_shipping
        fn get_by_base_product_id(&self, base_product_id: BaseProductId) -> RepoResult<LocalShipping> {
            Ok(LocalShipping {
                id: 1,
                base_product_id: base_product_id,
                store_id: StoreId(1),
                pickup: false,
                pickup_price: None,
                companies: vec![],
            })
        }

        /// Update a local_shipping
        fn update(&self, base_product_id_arg: BaseProductId, payload: UpdateLocalShipping) -> RepoResult<LocalShipping> {
            Ok(LocalShipping {
                id: 1,
                base_product_id: base_product_id_arg,
                store_id: StoreId(1),
                pickup: payload.pickup.unwrap_or_default(),
                pickup_price: payload.pickup_price,
                companies: payload.companies.unwrap_or_default(),
            })
        }

        /// Delete a local_shipping
        fn delete(&self, base_product_id_arg: BaseProductId) -> RepoResult<LocalShipping> {
            Ok(LocalShipping {
                id: 1,
                base_product_id: base_product_id_arg,
                store_id: StoreId(1),
                pickup: false,
                pickup_price: None,
                companies: vec![],
            })
        }
    }

    #[derive(Clone, Default)]
    pub struct CountriesRepoMock;

    impl CountriesRepo for CountriesRepoMock {
        /// Find specific country by label
        fn find(&self, label_arg: String) -> RepoResult<Option<Country>> {
            Ok(Some(Country {
                label: label_arg,
                name: vec![],
                children: vec![],
                level: 3,
                parent_label: Some("EEE".to_string()),
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
            label: "rus".to_string(),
            name: vec![],
            children: vec![],
            level: 3,
            parent_label: Some("EEE".to_string()),
        };
        let country_2 = Country {
            label: "EEE".to_string(),
            name: vec![],
            children: vec![country_3],
            level: 2,
            parent_label: Some("ALL".to_string()),
        };
        let country_1 = Country {
            label: "ALL".to_string(),
            name: vec![],
            children: vec![country_2],
            level: 1,
            parent_label: Some("root".to_string()),
        };
        Country {
            children: vec![country_1],
            ..Default::default()
        }
    }

    #[derive(Clone, Default)]
    pub struct RestrictionsRepoMock;

    impl RestrictionsRepo for RestrictionsRepoMock {
        fn create(&self, payload: NewRestriction) -> RepoResult<Restriction> {
            Ok(Restriction {
                id: 1,
                name: payload.name,
                max_weight: payload.max_weight,
                max_size: payload.max_size,
            })
        }

        fn get_by_name(&self, name: String) -> RepoResult<Restriction> {
            Ok(Restriction {
                id: 1,
                name: name,
                max_weight: 0f64,
                max_size: 0f64,
            })
        }

        fn update(&self, payload: UpdateRestriction) -> RepoResult<Restriction> {
            Ok(Restriction {
                id: 1,
                name: payload.name,
                max_weight: payload.max_weight.unwrap_or_default(),
                max_size: payload.max_size.unwrap_or_default(),
            })
        }

        fn delete(&self, name: String) -> RepoResult<Restriction> {
            Ok(Restriction {
                id: 1,
                name: name,
                max_weight: 0f64,
                max_size: 0f64,
            })
        }
    }

    #[derive(Clone, Default)]
    pub struct DeliveryToRepoMock;

    impl DeliveryToRepo for DeliveryToRepoMock {
        fn create(&self, payload: NewDeliveryTo) -> RepoResult<DeliveryTo> {
            Ok(DeliveryTo {
                id: 1,
                company_id: payload.company_id,
                country: payload.country,
                additional_info: payload.additional_info,
            })
        }

        fn list_by_company(&self, from: DeliveryCompany) -> RepoResult<Vec<DeliveryTo>> {
            Ok(vec![
                DeliveryTo {
                    id: 1,
                    company_id: from.clone(),
                    country: "US".to_string(),
                    additional_info: None,
                },
                DeliveryTo {
                    id: 2,
                    company_id: from.clone(),
                    country: "UK".to_string(),
                    additional_info: None,
                },
            ])
        }

        fn list_by_country(&self, from: String) -> RepoResult<Vec<DeliveryTo>> {
            Ok(vec![
                DeliveryTo {
                    id: 1,
                    company_id: DeliveryCompany::DHL,
                    country: from.clone(),
                    additional_info: None,
                },
                DeliveryTo {
                    id: 2,
                    company_id: DeliveryCompany::UPS,
                    country: from.clone(),
                    additional_info: None,
                },
            ])
        }

        fn update(&self, payload: UpdateDeliveryTo) -> RepoResult<DeliveryTo> {
            Ok(DeliveryTo {
                id: 1,
                company_id: payload.company_id,
                country: payload.country,
                additional_info: payload.additional_info,
            })
        }

        fn delete(&self, company_id: DeliveryCompany, country: String) -> RepoResult<DeliveryTo> {
            Ok(DeliveryTo {
                id: 1,
                company_id,
                country,
                additional_info: None,
            })
        }
    }

    #[derive(Clone, Default)]
    pub struct DeliveryFromRepoMock;

    impl DeliveryFromRepo for DeliveryFromRepoMock {
        fn create(&self, payload: NewDeliveryFrom) -> RepoResult<DeliveryFrom> {
            Ok(DeliveryFrom {
                id: 1,
                company_id: payload.company_id,
                country: payload.country,
                restriction_name: payload.restriction_name,
            })
        }

        fn list_by_company(&self, from: DeliveryCompany) -> RepoResult<Vec<DeliveryFrom>> {
            Ok(vec![
                DeliveryFrom {
                    id: 1,
                    company_id: from.clone(),
                    country: "US".to_string(),
                    restriction_name: format!("{}_{}", from, "US".to_string()),
                },
                DeliveryFrom {
                    id: 2,
                    company_id: from.clone(),
                    country: "UK".to_string(),
                    restriction_name: format!("{}_{}", from, "US".to_string()),
                },
            ])
        }

        fn update(&self, payload: UpdateDeliveryFrom) -> RepoResult<DeliveryFrom> {
            Ok(DeliveryFrom {
                id: 1,
                company_id: payload.company_id,
                country: payload.country,
                restriction_name: payload.restriction_name,
            })
        }

        fn delete(&self, company_id: DeliveryCompany, country: String) -> RepoResult<DeliveryFrom> {
            Ok(DeliveryFrom {
                id: 1,
                company_id,
                country,
                restriction_name: "".to_string(),
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
