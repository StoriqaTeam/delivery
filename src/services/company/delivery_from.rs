//! DeliveryFrom Service, presents CRUD operations
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Fail;
use futures::future::*;
use futures_cpupool::CpuPool;
use r2d2::{ManageConnection, Pool};

use stq_static_resources::DeliveryCompany;
use stq_types::UserId;

use errors::Error;
use models::company::{DeliveryFrom, NewDeliveryFrom, UpdateDeliveryFrom};
use repos::ReposFactory;
use services::types::ServiceFuture;

pub trait DeliveryFromService {
    /// Creates new delivery_from
    fn create(&self, payload: NewDeliveryFrom) -> ServiceFuture<DeliveryFrom>;

    /// Returns list of deliveries supported by the company, limited by `from` parameter
    fn list_by_company(&self, from: DeliveryCompany) -> ServiceFuture<Vec<DeliveryFrom>>;

    /// Update a delivery_from
    fn update(&self, payload: UpdateDeliveryFrom) -> ServiceFuture<DeliveryFrom>;

    /// Delete a delivery_from
    fn delete(&self, company_id: DeliveryCompany, country: String) -> ServiceFuture<DeliveryFrom>;
}

/// DeliveryFrom services, responsible for CRUD operations
pub struct DeliveryFromServiceImpl<
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
    > DeliveryFromServiceImpl<T, M, F>
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
    > DeliveryFromService for DeliveryFromServiceImpl<T, M, F>
{
    fn create(&self, payload: NewDeliveryFrom) -> ServiceFuture<DeliveryFrom> {
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
                            let delivery_from_repo = repo_factory.create_delivery_from_repo(&*conn, user_id);
                            delivery_from_repo.create(payload)
                        })
                })
                .map_err(|e| e.context("Service DeliveryFrom, create endpoint error occured.").into()),
        )
    }

    fn list_by_company(&self, from: DeliveryCompany) -> ServiceFuture<Vec<DeliveryFrom>> {
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
                            let delivery_from_repo = repo_factory.create_delivery_from_repo(&*conn, user_id);
                            delivery_from_repo.list_by_company(from)
                        })
                })
                .map_err(|e| e.context("Service DeliveryFrom, list_by_company endpoint error occured.").into()),
        )
    }

    fn update(&self, payload: UpdateDeliveryFrom) -> ServiceFuture<DeliveryFrom> {
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
                            let delivery_from_repo = repo_factory.create_delivery_from_repo(&*conn, user_id);
                            delivery_from_repo.update(payload)
                        })
                })
                .map_err(|e| e.context("Service DeliveryFrom, update endpoint error occured.").into()),
        )
    }

    fn delete(&self, company_id: DeliveryCompany, country: String) -> ServiceFuture<DeliveryFrom> {
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
                            let delivery_from_repo = repo_factory.create_delivery_from_repo(&*conn, user_id);
                            delivery_from_repo.delete(company_id, country)
                        })
                })
                .map_err(|e| e.context("Service DeliveryFrom, delete endpoint error occured.").into()),
        )
    }
}
