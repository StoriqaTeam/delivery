//! REPO LocalShipping table. LocalShipping is an entity that
//! describes the limits of the delivery company on
//! the dimensions of the goods.

use diesel;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::LoadQuery;
use diesel::query_dsl::RunQueryDsl;
use diesel::Connection;
use failure::Error as FailureError;

use stq_types::{BaseProductId, UserId};

use models::authorization::*;
use models::roles::UserRole;
use models::shipping::{LocalShipping, LocalShippingRaw, NewLocalShipping, UpdateLocalShipping};
use repos::legacy_acl::*;
use repos::types::RepoResult;
use repos::*;
use schema::local_shipping::dsl::*;
use schema::roles::dsl as Roles;

/// LocalShipping repository for handling LocalShipping
pub trait LocalShippingRepo {
    /// Create a new local_shipping
    fn create(&self, payload: NewLocalShipping) -> RepoResult<LocalShipping>;

    /// Get a local_shipping
    fn get_by_base_product_id(&self, base_product_id: BaseProductId) -> RepoResult<LocalShipping>;

    /// Update a local_shipping
    fn update(&self, base_product_id_arg: BaseProductId, payload: UpdateLocalShipping) -> RepoResult<LocalShipping>;

    /// Delete a local_shipping
    fn delete(&self, base_product_id_arg: BaseProductId) -> RepoResult<LocalShipping>;
}

pub struct LocalShippingRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<Acl<Resource, Action, Scope, FailureError, LocalShipping>>,
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> LocalShippingRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<Acl<Resource, Action, Scope, FailureError, LocalShipping>>) -> Self {
        Self { db_conn, acl }
    }

    fn execute_query<Ty: Send + 'static, U: LoadQuery<T, Ty> + Send + 'static>(&self, query: U) -> RepoResult<Ty> {
        query.get_result::<Ty>(self.db_conn).map_err(From::from)
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> LocalShippingRepo
    for LocalShippingRepoImpl<'a, T>
{
    fn create(&self, payload: NewLocalShipping) -> RepoResult<LocalShipping> {
        debug!("create new local_shipping {:?}.", payload);
        let payload = payload.to_raw()?;
        let query = diesel::insert_into(local_shipping).values(&payload);
        query
            .get_result::<LocalShippingRaw>(self.db_conn)
            .map_err(From::from)
            .and_then(|shipping| {
                let shipping = LocalShipping::from_raw(shipping)?;
                acl::check(&*self.acl, Resource::LocalShipping, Action::Create, self, Some(&shipping))?;
                Ok(shipping)
            })
            .map_err(|e: FailureError| e.context(format!("create new local_shipping {:?}.", payload)).into())
    }

    fn get_by_base_product_id(&self, base_product_id_arg: BaseProductId) -> RepoResult<LocalShipping> {
        debug!("get local_shipping by base_product_id {:?}.", base_product_id_arg);
        self.execute_query(local_shipping.filter(base_product_id.eq(base_product_id_arg)))
            .and_then(|shipping| {
                let shipping = LocalShipping::from_raw(shipping)?;
                acl::check(&*self.acl, Resource::LocalShipping, Action::Read, self, Some(&shipping))?;
                Ok(shipping)
            })
            .map_err(|e: FailureError| {
                e.context(format!(
                    "Getting local_shipping with base_product_id {:?} failed.",
                    base_product_id_arg
                )).into()
            })
    }

    fn update(&self, base_product_id_arg: BaseProductId, payload: UpdateLocalShipping) -> RepoResult<LocalShipping> {
        debug!("Updating local_shipping payload {:?}.", payload);
        let payload = payload.to_raw()?;
        self.execute_query(local_shipping.filter(base_product_id.eq(base_product_id_arg)))
            .and_then(LocalShipping::from_raw)
            .and_then(|shipping: LocalShipping| acl::check(&*self.acl, Resource::LocalShipping, Action::Update, self, Some(&shipping)))
            .and_then(|_| {
                let filter = local_shipping.filter(base_product_id.eq(base_product_id_arg));

                let query = diesel::update(filter).set(&payload);
                query.get_result::<LocalShippingRaw>(self.db_conn).map_err(From::from)
            })
            .and_then(LocalShipping::from_raw)
            .map_err(|e: FailureError| e.context(format!("Updating local_shipping payload {:?} failed.", payload)).into())
    }

    fn delete(&self, base_product_id_arg: BaseProductId) -> RepoResult<LocalShipping> {
        debug!("delete local_shipping {:?}.", base_product_id_arg);
        self.execute_query(local_shipping.filter(base_product_id.eq(base_product_id_arg)))
            .and_then(LocalShipping::from_raw)
            .and_then(|shipping: LocalShipping| acl::check(&*self.acl, Resource::LocalShipping, Action::Delete, self, Some(&shipping)))
            .and_then(|_| {
                let filter = local_shipping.filter(base_product_id.eq(base_product_id_arg));

                let query = diesel::delete(filter);
                query.get_result::<LocalShippingRaw>(self.db_conn).map_err(From::from)
            })
            .and_then(LocalShipping::from_raw)
            .map_err(|e: FailureError| {
                e.context(format!(
                    "Delete local_shipping with base product id {:?} failed.",
                    base_product_id_arg
                )).into()
            })
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, LocalShipping>
    for LocalShippingRepoImpl<'a, T>
{
    fn is_in_scope(&self, user_id_arg: UserId, scope: &Scope, obj: Option<&LocalShipping>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => {
                if let Some(obj) = obj {
                    Roles::roles
                        .filter(Roles::user_id.eq(user_id_arg))
                        .get_results::<UserRole>(self.db_conn)
                        .map_err(From::from)
                        .map(|user_roles_arg| {
                            user_roles_arg
                                .iter()
                                .any(|user_role_arg| user_role_arg.data.clone().map(|data| data == obj.store_id.0).unwrap_or_default())
                        })
                        .unwrap_or_else(|_: FailureError| false)
                } else {
                    false
                }
            }
        }
    }
}
