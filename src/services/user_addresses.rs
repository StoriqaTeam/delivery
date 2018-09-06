//! UserAddress Services, presents CRUD operations with user_roles

use futures_cpupool::CpuPool;

use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Fail;
use futures::Future;
use r2d2::{ManageConnection, Pool};

use stq_types::UserId;

use super::types::ServiceFuture;
use errors::Error;
use models::{NewUserAddress, UpdateUserAddress, UserAddress};
use repos::ReposFactory;

pub trait UserAddressService {
    /// Returns list of user  address
    fn get_addresses(&self, user_id: UserId) -> ServiceFuture<Vec<UserAddress>>;
    /// Create a new user addresses
    fn create(&self, payload: NewUserAddress) -> ServiceFuture<UserAddress>;
    /// Update a user addresses
    fn update(&self, id: i32, payload: UpdateUserAddress) -> ServiceFuture<UserAddress>;
    /// Delete user addresses
    fn delete(&self, id: i32) -> ServiceFuture<UserAddress>;
}

/// UserAddress services, responsible for UserAddress-related CRUD operations
pub struct UserAddressServiceImpl<
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
    > UserAddressServiceImpl<T, M, F>
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
    > UserAddressService for UserAddressServiceImpl<T, M, F>
{
    /// Returns list of user  address
    fn get_addresses(&self, user_id: UserId) -> ServiceFuture<Vec<UserAddress>> {
        let db_pool = self.db_pool.clone();
        let repo_factory = self.repo_factory.clone();
        let curent_user_id = self.user_id;

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(Error::Connection).into())
                        .and_then(move |conn| {
                            let users_addresses_repo = repo_factory.create_users_addresses_repo(&*conn, curent_user_id);
                            users_addresses_repo.list_for_user(user_id)
                        })
                })
                .map_err(|e| e.context("Service UserAddress, get_addresses endpoint error occured.").into()),
        )
    }

    /// Delete user addresses
    fn delete(&self, id: i32) -> ServiceFuture<UserAddress> {
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
                            let users_addresses_repo = repo_factory.create_users_addresses_repo(&*conn, user_id);
                            users_addresses_repo.delete(id)
                        })
                })
                .map_err(|e| e.context("Service UserAddress, delete endpoint error occured.").into()),
        )
    }

    /// Create a new user addresses
    fn create(&self, payload: NewUserAddress) -> ServiceFuture<UserAddress> {
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
                            let users_addresses_repo = repo_factory.create_users_addresses_repo(&*conn, user_id);
                            users_addresses_repo.create(payload)
                        })
                })
                .map_err(|e| e.context("Service UserAddress, create endpoint error occured.").into()),
        )
    }

    /// Update a user addresses
    fn update(&self, id: i32, payload: UpdateUserAddress) -> ServiceFuture<UserAddress> {
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
                            let users_addresses_repo = repo_factory.create_users_addresses_repo(&*conn, user_id);
                            users_addresses_repo.update(id, payload)
                        })
                })
                .map_err(|e| e.context("Service UserAddress, update endpoint error occured.").into()),
        )
    }
}
