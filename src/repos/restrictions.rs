//! REPO Restrictions table. Restrictions is an entity that
//! describes the limits of the delivery company on
//! the dimensions of the goods.

use super::types::RepoResult;
use models::company::{NewRestriction, Restriction, UpdateRestriction};

use diesel;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::LoadQuery;
use diesel::query_dsl::RunQueryDsl;
use diesel::Connection;

use models::authorization::*;

use repos::legacy_acl::*;
use repos::*;

use failure::Error as FailureError;
use failure::Fail;

use stq_types::UserId;

use schema::restrictions::dsl::*;

/// Restrictions repository for handling Restrictions
pub trait RestrictionsRepo {
    /// Create a new restriction
    fn create(&self, payload: NewRestriction) -> RepoResult<Restriction>;

    /// Get a restriction
    fn get_by_name(&self, name: String) -> RepoResult<Restriction>;

    /// Update a restriction
    fn update(&self, payload: UpdateRestriction) -> RepoResult<Restriction>;

    /// Delete a restriction
    fn delete(&self, name: String) -> RepoResult<Restriction>;
}

/// Implementation of UserRoles trait
pub struct RestrictionsRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<Acl<Resource, Action, Scope, FailureError, Restriction>>,
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> RestrictionsRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<Acl<Resource, Action, Scope, FailureError, Restriction>>) -> Self {
        Self { db_conn, acl }
    }

    fn execute_query<Ty: Send + 'static, U: LoadQuery<T, Ty> + Send + 'static>(&self, query: U) -> RepoResult<Ty> {
        query.get_result::<Ty>(self.db_conn).map_err(From::from)
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> RestrictionsRepo
    for RestrictionsRepoImpl<'a, T>
{
    fn create(&self, payload: NewRestriction) -> RepoResult<Restriction> {
        debug!("create new restriction {:?}.", payload);
        let query = diesel::insert_into(restrictions).values(&payload);
        query
            .get_result::<Restriction>(self.db_conn)
            .map_err(From::from)
            .and_then(|restriction| {
                acl::check(&*self.acl, Resource::Restrictions, Action::Create, self, Some(&restriction)).and_then(|_| Ok(restriction))
            })
            .map_err(|e: FailureError| e.context(format!("create new restriction {:?}.", payload)).into())
    }

    fn get_by_name(&self, name_: String) -> RepoResult<Restriction> {
        debug!("get restriction by name {:?}.", name_);
        self.execute_query(restrictions.filter(name.eq(name_.clone())))
            .and_then(|restriction| {
                acl::check(&*self.acl, Resource::Restrictions, Action::Read, self, Some(&restriction)).map(|_| restriction)
            })
            .map_err(|e: FailureError| e.context(format!("Getting restriction with name {:?} failed.", name_)).into())
    }

    fn update(&self, payload: UpdateRestriction) -> RepoResult<Restriction> {
        debug!("Updating restriction payload {:?}.", payload);
        self.execute_query(restrictions.filter(name.eq(payload.name.clone())))
            .and_then(|restriction: Restriction| acl::check(&*self.acl, Resource::Restrictions, Action::Update, self, Some(&restriction)))
            .and_then(|_| {
                let filter = restrictions.filter(name.eq(payload.name.clone()));

                let query = diesel::update(filter).set(&payload);
                query.get_result::<Restriction>(self.db_conn).map_err(From::from)
            })
            .map_err(|e: FailureError| e.context(format!("Updating restrictionpayload {:?} failed.", payload)).into())
    }

    fn delete(&self, name_: String) -> RepoResult<Restriction> {
        debug!("delete restriction {:?}.", name);
        acl::check(&*self.acl, Resource::Restrictions, Action::Delete, self, None)?;
        let filter = restrictions.filter(name.eq(name_.clone()));
        let query = diesel::delete(filter);
        query
            .get_result(self.db_conn)
            .map_err(move |e| e.context(format!("delete restriction {:?}.", name_)).into())
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, Restriction>
    for RestrictionsRepoImpl<'a, T>
{
    fn is_in_scope(&self, _user_id: UserId, scope: &Scope, _obj: Option<&Restriction>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => false,
        }
    }
}
