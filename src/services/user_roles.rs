//! UserRoles Services, presents CRUD operations with user_roles

use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Error as FailureError;

use r2d2::ManageConnection;

use stq_types::{DeliveryRole, RoleId, UserId};

use super::types::{Service, ServiceFuture};
use models::{NewUserRole, UserRole};
use repos::ReposFactory;

pub trait UserRolesService {
    /// Creates new user_role
    fn create_role(&self, payload: NewUserRole) -> ServiceFuture<UserRole>;
    /// Returns role by user ID
    fn get_roles(&self, user_id: UserId) -> ServiceFuture<Vec<DeliveryRole>>;
    /// Deletes roles for user
    fn delete_by_user_id(&self, user_id_arg: UserId) -> ServiceFuture<Vec<UserRole>>;
    /// Deletes role for user by id
    fn delete_by_id(&self, id_arg: RoleId) -> ServiceFuture<UserRole>;
}
impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > UserRolesService for Service<T, M, F>
{
    /// Returns role by user ID
    fn get_roles(&self, user_id: UserId) -> ServiceFuture<Vec<DeliveryRole>> {
        let repo_factory = self.static_context.repo_factory.clone();
        let current_uid = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let user_roles_repo = repo_factory.create_user_roles_repo(&*conn, current_uid);
            user_roles_repo
                .list_for_user(user_id)
                .map_err(|e: FailureError| e.context("Service user_roles, get_roles endpoint error occured.").into())
        })
    }

    /// Deletes role for user by id
    fn delete_by_id(&self, id_arg: RoleId) -> ServiceFuture<UserRole> {
        let repo_factory = self.static_context.repo_factory.clone();
        let current_uid = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let user_roles_repo = repo_factory.create_user_roles_repo(&*conn, current_uid);
            user_roles_repo
                .delete_by_id(id_arg)
                .map_err(|e: FailureError| e.context("Service user_roles, delete_by_id endpoint error occured.").into())
        })
    }

    /// Deletes specific user role
    fn delete_by_user_id(&self, user_id_arg: UserId) -> ServiceFuture<Vec<UserRole>> {
        let repo_factory = self.static_context.repo_factory.clone();
        let current_uid = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let user_roles_repo = repo_factory.create_user_roles_repo(&*conn, current_uid);
            user_roles_repo
                .delete_by_user_id(user_id_arg)
                .map_err(|e: FailureError| e.context("Service user_roles, delete_by_user_id endpoint error occured.").into())
        })
    }

    /// Creates new user_role
    fn create_role(&self, new_user_role: NewUserRole) -> ServiceFuture<UserRole> {
        let repo_factory = self.static_context.repo_factory.clone();
        let current_uid = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let user_roles_repo = repo_factory.create_user_roles_repo(&*conn, current_uid);
            conn.transaction::<UserRole, FailureError, _>(move || user_roles_repo.create(new_user_role))
                .map_err(|e: FailureError| e.context("Service user_roles, create endpoint error occured.").into())
        })
    }
}
