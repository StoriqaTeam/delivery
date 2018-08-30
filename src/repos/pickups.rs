//! Repo Pickups table.

use diesel;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::LoadQuery;
use diesel::query_dsl::RunQueryDsl;
use diesel::Connection;

use failure::Error as FailureError;
use failure::Fail;

use stq_types::{BaseProductId, UserId};

use models::authorization::*;
use repos::legacy_acl::*;
use repos::types::RepoResult;

use models::pickups::{NewPickups, Pickups, UpdatePickups};
use repos::acl;
use schema::pickups::dsl::*;

/// pickups repository for handling pickups model
pub trait PickupsRepo {
    /// Create a new pickups
    fn create(&self, payload: NewPickups) -> RepoResult<Pickups>;

    /// Update a pickups
    fn update(&self, base_product_id_arg: BaseProductId, payload: UpdatePickups) -> RepoResult<Pickups>;

    /// Delete a pickups
    fn delete(&self, id_arg: i32) -> RepoResult<Pickups>;
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
        query.get_result::<Ty>(self.db_conn).map_err(From::from)
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> PickupsRepo for PickupsRepoImpl<'a, T> {
    fn create(&self, payload: NewPickups) -> RepoResult<Pickups> {
        debug!("create new pickups {:?}.", payload);
        let query = diesel::insert_into(pickups).values(&payload);
        query
            .get_result::<Pickups>(self.db_conn)
            .map_err(From::from)
            .and_then(|record| acl::check(&*self.acl, Resource::Companies, Action::Create, self, Some(&record)).and_then(|_| Ok(record)))
            .map_err(|e: FailureError| e.context(format!("create new pickups {:?}.", payload)).into())
    }

    /// Update a pickups
    fn update(&self, base_product_id_arg: BaseProductId, payload: UpdatePickups) -> RepoResult<Pickups> {
        debug!("Updating pickups payload {:?}.", payload);
        self.execute_query(pickups.filter(base_product_id.eq(base_product_id_arg)))
            .and_then(|pickup_: Pickups| acl::check(&*self.acl, Resource::Pickups, Action::Update, self, Some(&pickup_)))
            .and_then(|_| {
                let filtered = pickups.filter(base_product_id.eq(base_product_id_arg));
                let query = diesel::update(filtered).set(&payload);
                query.get_result::<Pickups>(self.db_conn).map_err(From::from)
            })
            .map_err(|e: FailureError| e.context(format!("Updating products payload {:?} failed.", payload)).into())
    }

    fn delete(&self, id_arg: i32) -> RepoResult<Pickups> {
        debug!("delete pickups by id: {}.", id_arg);

        acl::check(&*self.acl, Resource::Pickups, Action::Delete, self, None)?;
        let filtered = pickups.filter(id.eq(id_arg.clone()));
        let query = diesel::delete(filtered);
        query
            .get_result(self.db_conn)
            .map_err(move |e| e.context(format!("delete pickups id: {}.", id_arg)).into())
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, Pickups>
    for PickupsRepoImpl<'a, T>
{
    fn is_in_scope(&self, _user_id: UserId, scope: &Scope, _obj: Option<&Pickups>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => false,
        }
    }
}
