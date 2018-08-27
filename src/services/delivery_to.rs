//! DeliveryTo Service, presents CRUD operations
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
use models::company::{DeliveryTo, NewDeliveryTo, UpdateDeliveryTo};
use repos::ReposFactory;
use stq_static_resources::DeliveryCompany;

pub trait DeliveryToService {
    /// Creates new delivery_to
    fn create(&self, payload: NewDeliveryTo) -> ServiceFuture<DeliveryTo>;

    /// Returns list of deliveries supported by the company, limited by `from` parameter
    fn list_by_company(&self, from: DeliveryCompany) -> ServiceFuture<Vec<DeliveryTo>>;

    /// Returns list of deliveries supported by the country, limited by `from` parameter
    fn list_by_country(&self, from: String) -> ServiceFuture<Vec<DeliveryTo>>;

    /// Update a delivery_to
    fn update(&self, payload: UpdateDeliveryTo) -> ServiceFuture<DeliveryTo>;

    /// Delete a delivery_to
    fn delete(&self, company_id: DeliveryCompany, country: String) -> ServiceFuture<DeliveryTo>;
}

/// DeliveryTo services, responsible for CRUD operations
pub struct DeliveryToServiceImpl<
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
    > DeliveryToServiceImpl<T, M, F>
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
    > DeliveryToService for DeliveryToServiceImpl<T, M, F>
{
    fn create(&self, payload: NewDeliveryTo) -> ServiceFuture<DeliveryTo> {
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
                            let delivery_to_repo = repo_factory.create_delivery_to_repo(&*conn, user_id);
                            delivery_to_repo.create(payload)
                        })
                })
                .map_err(|e| e.context("Service DeliveryTo, create endpoint error occured.").into()),
        )
    }

    fn list_by_company(&self, from: DeliveryCompany) -> ServiceFuture<Vec<DeliveryTo>> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let repo_factory = self.repo_factory.clone();

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(Error::Connection).into())
                        .and_then(move |conn| {
                            let delivery_to_repo = repo_factory.create_delivery_to_repo(&*conn, user_id);
                            delivery_to_repo.list_by_company(from)
                        })
                })
                .map_err(|e| e.context("Service DeliveryTo, list_by_company endpoint error occured.").into()),
        )
    }

    fn list_by_country(&self, from: String) -> ServiceFuture<Vec<DeliveryTo>> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let repo_factory = self.repo_factory.clone();

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(Error::Connection).into())
                        .and_then(move |conn| {
                            let delivery_to_repo = repo_factory.create_delivery_to_repo(&*conn, user_id);
                            delivery_to_repo.list_by_country(from)
                        })
                })
                .map_err(|e| e.context("Service DeliveryTo, list_by_country endpoint error occured.").into()),
        )
    }

    fn update(&self, payload: UpdateDeliveryTo) -> ServiceFuture<DeliveryTo> {
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
                            let delivery_to_repo = repo_factory.create_delivery_to_repo(&*conn, user_id);
                            delivery_to_repo.update(payload)
                        })
                })
                .map_err(|e| e.context("Service DeliveryTo, update endpoint error occured.").into()),
        )
    }

    fn delete(&self, company_id: DeliveryCompany, country: String) -> ServiceFuture<DeliveryTo> {
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
                            let delivery_to_repo = repo_factory.create_delivery_to_repo(&*conn, user_id);
                            delivery_to_repo.delete(company_id, country)
                        })
                })
                .map_err(|e| e.context("Service DeliveryTo, delete endpoint error occured.").into()),
        )
    }
}
