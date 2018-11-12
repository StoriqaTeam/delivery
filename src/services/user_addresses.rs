//! UserAddress Services, presents CRUD operations with user_roles

use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;

use r2d2::ManageConnection;

use failure::Error as FailureError;

use stq_types::UserId;

use super::types::{Service, ServiceFuture};
use models::{NewUserAddress, UpdateUserAddress, UserAddress};
use repos::ReposFactory;

pub trait UserAddressService {
    /// Returns list of user  address
    fn get_addresses(&self, user_id: UserId) -> ServiceFuture<Vec<UserAddress>>;
    /// Create a new user addresses
    fn create_address(&self, payload: NewUserAddress) -> ServiceFuture<UserAddress>;
    /// Update a user addresses
    fn update_address(&self, id: i32, payload: UpdateUserAddress) -> ServiceFuture<UserAddress>;
    /// Delete user addresses
    fn delete_address(&self, id: i32) -> ServiceFuture<UserAddress>;
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > UserAddressService for Service<T, M, F>
{
    /// Returns list of user  address
    fn get_addresses(&self, user_id: UserId) -> ServiceFuture<Vec<UserAddress>> {
        let repo_factory = self.static_context.repo_factory.clone();
        let current_user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let users_addresses_repo = repo_factory.create_users_addresses_repo(&*conn, current_user_id);
            users_addresses_repo
                .list_for_user(user_id)
                .map_err(|e| e.context("Service UserAddress, get_addresses endpoint error occured.").into())
        })
    }

    /// Delete user addresses
    fn delete_address(&self, id: i32) -> ServiceFuture<UserAddress> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let users_addresses_repo = repo_factory.create_users_addresses_repo(&*conn, user_id);
            users_addresses_repo
                .delete(id)
                .map_err(|e| e.context("Service UserAddress, delete endpoint error occured.").into())
        })
    }

    /// Create a new user addresses
    fn create_address(&self, payload: NewUserAddress) -> ServiceFuture<UserAddress> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let users_addresses_repo = repo_factory.create_users_addresses_repo(&*conn, user_id);
            conn.transaction::<UserAddress, FailureError, _>(move || {
                users_addresses_repo
                    .create(payload)
                    .map_err(|e| e.context("Service UserAddress, create endpoint error occured.").into())
            })
        })
    }

    /// Update a user addresses
    fn update_address(&self, id: i32, payload: UpdateUserAddress) -> ServiceFuture<UserAddress> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let users_addresses_repo = repo_factory.create_users_addresses_repo(&*conn, user_id);
            users_addresses_repo
                .update(id, payload)
                .map_err(|e| e.context("Service UserAddress, update endpoint error occured.").into())
        })
    }
}
