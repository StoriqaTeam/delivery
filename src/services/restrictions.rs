//! Restriction Service, presents CRUD operations
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Fail;
use futures::future::*;
use futures_cpupool::CpuPool;
use r2d2::{ManageConnection, Pool};

use errors::Error;
use stq_types::UserId;

use super::types::ServiceFuture;
use models::company::{NewRestriction, Restriction, UpdateRestriction};
use repos::ReposFactory;

pub trait RestrictionService {
    /// Creates new restriction
    fn create(&self, payload: NewRestriction) -> ServiceFuture<Restriction>;

    /// Get a restriction
    fn get_by_name(&self, restriction_name: String) -> ServiceFuture<Restriction>;

    /// Update a restriction
    fn update(&self, payload: UpdateRestriction) -> ServiceFuture<Restriction>;

    /// Delete a restriction
    fn delete(&self, restriction_name: String) -> ServiceFuture<Restriction>;
}

/// Restriction services, responsible for UserRole-related CRUD operations
pub struct RestrictionServiceImpl<
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    M: ManageConnection<Connection = T>,
    F: ReposFactory<T>,
> {
    pub db_pool: Pool<M>,
    pub cpu_pool: CpuPool,
    pub user_id: Option<UserId>,
    pub repo_factory: F,
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > RestrictionServiceImpl<T, M, F>
{
    pub fn new(db_pool: Pool<M>, cpu_pool: CpuPool, user_id: Option<UserId>, repo_factory: F) -> Self {
        Self {
            db_pool,
            cpu_pool,
            user_id,
            repo_factory,
        }
    }
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > RestrictionService for RestrictionServiceImpl<T, M, F>
{
    fn create(&self, payload: NewRestriction) -> ServiceFuture<Restriction> {
        let db_pool = self.db_pool.clone();
        let repo_factory = self.repo_factory.clone();
        let user_id = self.user_id;

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(Error::Connection).into())
                        .and_then(move |conn| {
                            let restrictions_repo = repo_factory.create_restrictions_repo(&*conn, user_id);
                            restrictions_repo.create(payload)
                        })
                })
                .map_err(|e| e.context("Service Restrictions, create endpoint error occured.").into()),
        )
    }

    fn get_by_name(&self, restriction_name: String) -> ServiceFuture<Restriction> {
        let db_pool = self.db_pool.clone();
        let repo_factory = self.repo_factory.clone();
        let user_id = self.user_id;

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(Error::Connection).into())
                        .and_then(move |conn| {
                            let restrictions_repo = repo_factory.create_restrictions_repo(&*conn, user_id);
                            restrictions_repo.get_by_name(restriction_name)
                        })
                })
                .map_err(|e| e.context("Service Restrictions, get_by_name endpoint error occured.").into()),
        )
    }

    fn update(&self, payload: UpdateRestriction) -> ServiceFuture<Restriction> {
        let db_pool = self.db_pool.clone();
        let repo_factory = self.repo_factory.clone();
        let user_id = self.user_id;

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(Error::Connection).into())
                        .and_then(move |conn| {
                            let restrictions_repo = repo_factory.create_restrictions_repo(&*conn, user_id);
                            restrictions_repo.update(payload)
                        })
                })
                .map_err(|e| e.context("Service Restrictions, update endpoint error occured.").into()),
        )
    }

    fn delete(&self, restriction_name: String) -> ServiceFuture<Restriction> {
        let db_pool = self.db_pool.clone();
        let repo_factory = self.repo_factory.clone();
        let user_id = self.user_id;

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(Error::Connection).into())
                        .and_then(move |conn| {
                            let restrictions_repo = repo_factory.create_restrictions_repo(&*conn, user_id);
                            restrictions_repo.delete(restriction_name)
                        })
                })
                .map_err(|e| e.context("Service Restrictions, delete endpoint error occured.").into()),
        )
    }
}
