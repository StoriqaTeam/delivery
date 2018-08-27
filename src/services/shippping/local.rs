//! LocalShipping Service, presents CRUD operations
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Fail;
use futures::future::*;
use futures_cpupool::CpuPool;
use r2d2::{ManageConnection, Pool};

use stq_types::{BaseProductId, UserId};

use errors::Error;
use models::{LocalShipping, NewLocalShipping, UpdateLocalShipping};
use repos::ReposFactory;
use services::types::ServiceFuture;

pub trait LocalShippingService {
    /// Creates new local_shipping
    fn create(&self, payload: NewLocalShipping) -> ServiceFuture<LocalShipping>;

    /// Get a local_shipping
    fn get_by_base_product_id(&self, base_product_id: BaseProductId) -> ServiceFuture<LocalShipping>;

    /// Update a local_shipping
    fn update(&self, base_product_id_arg: BaseProductId, payload: UpdateLocalShipping) -> ServiceFuture<LocalShipping>;

    /// Delete a local_shipping
    fn delete(&self, base_product_id_arg: BaseProductId) -> ServiceFuture<LocalShipping>;
}

/// LocalShipping services, responsible for CRUD operations
pub struct LocalShippingServiceImpl<
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
    > LocalShippingServiceImpl<T, M, F>
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
    > LocalShippingService for LocalShippingServiceImpl<T, M, F>
{
    fn create(&self, payload: NewLocalShipping) -> ServiceFuture<LocalShipping> {
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
                            let local_shippings_repo = repo_factory.create_local_shippings_repo(&*conn, user_id);
                            local_shippings_repo.create(payload)
                        })
                })
                .map_err(|e| e.context("Service LocalShippings, create endpoint error occured.").into()),
        )
    }

    fn get_by_base_product_id(&self, base_product_id: BaseProductId) -> ServiceFuture<LocalShipping> {
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
                            let local_shippings_repo = repo_factory.create_local_shippings_repo(&*conn, user_id);
                            local_shippings_repo.get_by_base_product_id(base_product_id)
                        })
                })
                .map_err(|e| {
                    e.context("Service LocalShippings, get_by_base_product_id endpoint error occured.")
                        .into()
                }),
        )
    }

    fn update(&self, base_product_id_arg: BaseProductId, payload: UpdateLocalShipping) -> ServiceFuture<LocalShipping> {
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
                            let local_shippings_repo = repo_factory.create_local_shippings_repo(&*conn, user_id);
                            local_shippings_repo.update(base_product_id_arg, payload)
                        })
                })
                .map_err(|e| e.context("Service LocalShippings, update endpoint error occured.").into()),
        )
    }

    fn delete(&self, base_product_id_arg: BaseProductId) -> ServiceFuture<LocalShipping> {
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
                            let local_shippings_repo = repo_factory.create_local_shippings_repo(&*conn, user_id);
                            local_shippings_repo.delete(base_product_id_arg)
                        })
                })
                .map_err(|e| e.context("Service LocalShippings, delete endpoint error occured.").into()),
        )
    }
}
