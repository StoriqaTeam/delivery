//! Repo Pickups table. Pickups is an entity that
//! contains info about local shipping of base_product.

use diesel;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::LoadQuery;
use diesel::query_dsl::RunQueryDsl;
use diesel::Connection;

use errors::Error;
use failure::Error as FailureError;
use failure::Fail;

use stq_types::{BaseProductId, UserId};

use models::authorization::*;
use repos::legacy_acl::*;
use repos::types::RepoResult;

use models::pickups::{NewPickups, Pickups, UpdatePickups};
use models::roles::UserRole;
use repos::acl;
use schema::pickups::dsl::*;
use schema::roles::dsl as Roles;

/// pickups repository for handling pickups model
pub trait PickupsRepo {
    /// Create a new pickups
    fn create(&self, payload: NewPickups) -> RepoResult<Pickups>;

    /// Getting pickups
    fn list(&self) -> RepoResult<Vec<Pickups>>;

    /// Getting pickups by base_product_id
    fn get(&self, base_product_id_arg: BaseProductId) -> RepoResult<Option<Pickups>>;

    /// Update a pickups
    fn update(&self, base_product_id_arg: BaseProductId, payload: UpdatePickups) -> RepoResult<Pickups>;

    /// Delete a pickups
    fn delete(&self, base_product_id_arg: BaseProductId) -> RepoResult<Option<Pickups>>;
}

/// Implementation of PickupsRepo trait
pub struct PickupsRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<Acl<Resource, Action, Scope, FailureError, Pickups>>,
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> PickupsRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<Acl<Resource, Action, Scope, FailureError, Pickups>>) -> Self {
        Self { db_conn, acl }
    }

    fn execute_query<Ty: Send + 'static, U: LoadQuery<T, Ty> + Send + 'static>(&self, query: U) -> RepoResult<Ty> {
        query.get_result::<Ty>(self.db_conn).map_err(|e| Error::from(e).into())
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> PickupsRepo for PickupsRepoImpl<'a, T> {
    fn create(&self, payload: NewPickups) -> RepoResult<Pickups> {
        debug!("create new pickups {:?}.", payload);
        let query = diesel::insert_into(pickups).values(&payload);
        query
            .get_result::<Pickups>(self.db_conn)
            .map_err(|e| Error::from(e).into())
            .and_then(|record| acl::check(&*self.acl, Resource::Pickups, Action::Create, self, Some(&record)).and_then(|_| Ok(record)))
            .map_err(|e: FailureError| e.context(format!("create new pickups {:?}.", payload)).into())
    }

    /// Getting pickups
    fn list(&self) -> RepoResult<Vec<Pickups>> {
        debug!("List pickups");
        let query = pickups.order(id);

        query
            .get_results(self.db_conn)
            .map_err(|e| Error::from(e).into())
            .and_then(|results: Vec<Pickups>| {
                for result in &results {
                    acl::check(&*self.acl, Resource::Pickups, Action::Read, self, Some(&result))?;
                }
                Ok(results)
            })
            .map_err(|e: FailureError| e.context("Find in pickups error occured").into())
    }

    /// Getting pickups by base_product_id
    fn get(&self, base_product_id_arg: BaseProductId) -> RepoResult<Option<Pickups>> {
        debug!("Getting pickups by base_product_id {}", base_product_id_arg);
        let query = pickups.filter(base_product_id.eq(base_product_id_arg)).order(id);

        query
            .get_result(self.db_conn)
            .optional()
            .map_err(|e| Error::from(e).into())
            .and_then(|result: Option<Pickups>| {
                if let Some(ref result) = result {
                    acl::check(&*self.acl, Resource::Pickups, Action::Read, self, Some(result))?;
                }
                Ok(result)
            })
            .map_err(|e: FailureError| {
                e.context(format!("Getting pickups by base_product_id {}", base_product_id_arg))
                    .into()
            })
    }

    /// Update a pickups
    fn update(&self, base_product_id_arg: BaseProductId, payload: UpdatePickups) -> RepoResult<Pickups> {
        debug!("Updating pickups payload {:?}.", payload);
        self.execute_query(pickups.filter(base_product_id.eq(base_product_id_arg)))
            .and_then(|pickup_: Pickups| acl::check(&*self.acl, Resource::Pickups, Action::Update, self, Some(&pickup_)))
            .and_then(|_| {
                let filtered = pickups.filter(base_product_id.eq(base_product_id_arg));
                let query = diesel::update(filtered).set(&payload);
                query.get_result::<Pickups>(self.db_conn).map_err(|e| Error::from(e).into())
            })
            .map_err(|e: FailureError| e.context(format!("Updating products payload {:?} failed.", payload)).into())
    }

    fn delete(&self, base_product_id_arg: BaseProductId) -> RepoResult<Option<Pickups>> {
        debug!("delete pickups by base_product_id: {}.", base_product_id_arg);
        let query = pickups.filter(base_product_id.eq(base_product_id_arg)).order(id);

        query
            .get_result(self.db_conn)
            .optional()
            .map_err(|e| Error::from(e).into())
            .and_then(|pickup_: Option<Pickups>| {
                if let Some(ref pickup_) = pickup_ {
                    acl::check(&*self.acl, Resource::Pickups, Action::Delete, self, Some(pickup_))?;
                }
                Ok(pickup_)
            })
            .and_then(|_| {
                let filtered = pickups.filter(base_product_id.eq(base_product_id_arg));
                let query = diesel::delete(filtered);
                query.get_result(self.db_conn).optional().map_err(move |e| {
                    e.context(format!("delete pickups by base_product_id: {}", base_product_id_arg))
                        .into()
                })
            })
            .map_err(|e: FailureError| {
                e.context(format!("delete pickups by base_product_id: {} failed", base_product_id_arg))
                    .into()
            })
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, Pickups>
    for PickupsRepoImpl<'a, T>
{
    fn is_in_scope(&self, user_id_arg: UserId, scope: &Scope, obj: Option<&Pickups>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => {
                if let Some(obj) = obj {
                    Roles::roles
                        .filter(Roles::user_id.eq(user_id_arg))
                        .get_results::<UserRole>(self.db_conn)
                        .map_err(|e| Error::from(e).into())
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
