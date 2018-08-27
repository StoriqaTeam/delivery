//! REPO InternationalShipping table. InternationalShipping is an entity that
//! contains info about international shipping of base_product.

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
use models::shipping::{InternationalShipping, InternationalShippingRaw, NewInternationalShipping, UpdateInternationalShipping};
use repos::legacy_acl::*;
use repos::types::RepoResult;
use repos::*;
use schema::international_shipping::dsl::*;
use schema::roles::dsl as Roles;

/// InternationalShipping repository for handling InternationalShipping
pub trait InternationalShippingRepo {
    /// Create a new international_shipping
    fn create(&self, payload: NewInternationalShipping) -> RepoResult<InternationalShipping>;

    /// Get a international_shipping
    fn get_by_base_product_id(&self, base_product_id: BaseProductId) -> RepoResult<InternationalShipping>;

    /// Update a international_shipping
    fn update(&self, base_product_id_arg: BaseProductId, payload: UpdateInternationalShipping) -> RepoResult<InternationalShipping>;

    /// Delete a international_shipping
    fn delete(&self, base_product_id_arg: BaseProductId) -> RepoResult<InternationalShipping>;
}

pub struct InternationalShippingRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<Acl<Resource, Action, Scope, FailureError, InternationalShipping>>,
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> InternationalShippingRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<Acl<Resource, Action, Scope, FailureError, InternationalShipping>>) -> Self {
        Self { db_conn, acl }
    }

    fn execute_query<Ty: Send + 'static, U: LoadQuery<T, Ty> + Send + 'static>(&self, query: U) -> RepoResult<Ty> {
        query.get_result::<Ty>(self.db_conn).map_err(From::from)
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> InternationalShippingRepo
    for InternationalShippingRepoImpl<'a, T>
{
    fn create(&self, payload: NewInternationalShipping) -> RepoResult<InternationalShipping> {
        debug!("create new international_shipping {:?}.", payload);
        let payload = payload.to_raw()?;
        let query = diesel::insert_into(international_shipping).values(&payload);
        query
            .get_result::<InternationalShippingRaw>(self.db_conn)
            .map_err(From::from)
            .and_then(|shipping| {
                let shipping = InternationalShipping::from_raw(shipping)?;
                acl::check(&*self.acl, Resource::InternationalShipping, Action::Create, self, Some(&shipping))?;
                Ok(shipping)
            })
            .map_err(|e: FailureError| e.context(format!("create new international_shipping {:?}.", payload)).into())
    }

    fn get_by_base_product_id(&self, base_product_id_arg: BaseProductId) -> RepoResult<InternationalShipping> {
        debug!("get international_shipping by base_product_id {:?}.", base_product_id_arg);
        self.execute_query(international_shipping.filter(base_product_id.eq(base_product_id_arg)))
            .and_then(|shipping| {
                let shipping = InternationalShipping::from_raw(shipping)?;
                acl::check(&*self.acl, Resource::InternationalShipping, Action::Read, self, Some(&shipping))?;
                Ok(shipping)
            })
            .map_err(|e: FailureError| {
                e.context(format!(
                    "Getting international_shipping with base_product_id {:?} failed.",
                    base_product_id_arg
                )).into()
            })
    }

    fn update(&self, base_product_id_arg: BaseProductId, payload: UpdateInternationalShipping) -> RepoResult<InternationalShipping> {
        debug!("Updating international_shipping payload {:?}.", payload);
        let payload = payload.to_raw()?;
        self.execute_query(international_shipping.filter(base_product_id.eq(base_product_id_arg)))
            .and_then(InternationalShipping::from_raw)
            .and_then(|shipping: InternationalShipping| {
                acl::check(&*self.acl, Resource::InternationalShipping, Action::Update, self, Some(&shipping))
            })
            .and_then(|_| {
                let filter = international_shipping.filter(base_product_id.eq(base_product_id_arg));

                let query = diesel::update(filter).set(&payload);
                query.get_result::<InternationalShippingRaw>(self.db_conn).map_err(From::from)
            })
            .and_then(InternationalShipping::from_raw)
            .map_err(|e: FailureError| {
                e.context(format!("Updating international_shipping payload {:?} failed.", payload))
                    .into()
            })
    }

    fn delete(&self, base_product_id_arg: BaseProductId) -> RepoResult<InternationalShipping> {
        debug!("delete international_shipping {:?}.", base_product_id_arg);
        self.execute_query(international_shipping.filter(base_product_id.eq(base_product_id_arg)))
            .and_then(InternationalShipping::from_raw)
            .and_then(|shipping: InternationalShipping| {
                acl::check(&*self.acl, Resource::InternationalShipping, Action::Delete, self, Some(&shipping))
            })
            .and_then(|_| {
                let filter = international_shipping.filter(base_product_id.eq(base_product_id_arg));

                let query = diesel::delete(filter);
                query.get_result::<InternationalShippingRaw>(self.db_conn).map_err(From::from)
            })
            .and_then(InternationalShipping::from_raw)
            .map_err(|e: FailureError| {
                e.context(format!(
                    "Delete international_shipping with base product id {:?} failed.",
                    base_product_id_arg
                )).into()
            })
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, InternationalShipping>
    for InternationalShippingRepoImpl<'a, T>
{
    fn is_in_scope(&self, user_id_arg: UserId, scope: &Scope, obj: Option<&InternationalShipping>) -> bool {
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
