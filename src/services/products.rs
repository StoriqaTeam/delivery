//! Products Service, presents CRUD operations
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Fail;
use futures::future::*;
use futures_cpupool::CpuPool;
use r2d2::{ManageConnection, Pool};

use stq_types::{BaseProductId, UserId};

use errors::Error;
use models::{NewProducts, Products, UpdateProducts};
use repos::ReposFactory;
use services::types::ServiceFuture;

pub trait ProductsService {
    /// Creates new products
    fn create(&self, payload: NewProducts) -> ServiceFuture<Products>;

    /// Get  products
    fn get_by_base_product_id(&self, base_product_id: BaseProductId) -> ServiceFuture<Vec<Products>>;

    /// Update a product
    fn update(&self, base_product_id_arg: BaseProductId, company_package_id: i32, payload: UpdateProducts) -> ServiceFuture<Products>;

    /// Delete a products
    fn delete(&self, base_product_id_arg: BaseProductId) -> ServiceFuture<Products>;
}

/// Products services, responsible for CRUD operations
pub struct ProductsServiceImpl<
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
    > ProductsServiceImpl<T, M, F>
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
    > ProductsService for ProductsServiceImpl<T, M, F>
{
    fn create(&self, payload: NewProducts) -> ServiceFuture<Products> {
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
                            let products_repo = repo_factory.create_products_repo(&*conn, user_id);
                            products_repo.create(payload)
                        })
                })
                .map_err(|e| e.context("Service Products, create endpoint error occured.").into()),
        )
    }

    fn get_by_base_product_id(&self, base_product_id: BaseProductId) -> ServiceFuture<Vec<Products>> {
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
                            let products_repo = repo_factory.create_products_repo(&*conn, user_id);
                            products_repo.get_by_base_product_id(base_product_id)
                        })
                })
                .map_err(|e| e.context("Service Products, get_by_base_product_id endpoint error occured.").into()),
        )
    }

    fn update(&self, base_product_id_arg: BaseProductId, company_package_id: i32, payload: UpdateProducts) -> ServiceFuture<Products> {
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
                            let products_repo = repo_factory.create_products_repo(&*conn, user_id);
                            products_repo.update(base_product_id_arg, company_package_id, payload)
                        })
                })
                .map_err(|e| e.context("Service Products, update endpoint error occured.").into()),
        )
    }

    fn delete(&self, base_product_id_arg: BaseProductId) -> ServiceFuture<Products> {
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
                            let products_repo = repo_factory.create_products_repo(&*conn, user_id);
                            products_repo.delete(base_product_id_arg)
                        })
                })
                .map_err(|e| e.context("Service Products, delete endpoint error occured.").into()),
        )
    }
}
