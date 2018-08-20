//! UserRoles Services, presents CRUD operations with user_roles
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Fail;
use futures::future::*;
use futures_cpupool::CpuPool;
use r2d2::{ManageConnection, Pool};

use stq_types::{UserId, UsersRole};

use errors::Error;

use super::types::ServiceFuture;
use models::{NewUserRole, OldUserRole, UserRole};
use repos::ReposFactory;

pub trait UserRolesService {
    /// Returns role by user ID
    fn get_roles(&self, user_id: UserId) -> ServiceFuture<Vec<UsersRole>>;
    /// Delete specific user role
    fn delete(&self, payload: OldUserRole) -> ServiceFuture<UserRole>;
    /// Creates new user_role
    fn create(&self, payload: NewUserRole) -> ServiceFuture<UserRole>;
    /// Deletes default roles for user
    fn delete_default(&self, user_id: UserId) -> ServiceFuture<UserRole>;
    /// Creates default roles for user
    fn create_default(&self, user_id: UserId) -> ServiceFuture<UserRole>;
}

/// UserRoles services, responsible for UserRole-related CRUD operations
pub struct UserRolesServiceImpl<
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    M: ManageConnection<Connection = T>,
    F: ReposFactory<T>,
> {
    pub db_pool: Pool<M>,
    pub cpu_pool: CpuPool,
    pub repo_factory: F,
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > UserRolesServiceImpl<T, M, F>
{
    pub fn new(db_pool: Pool<M>, cpu_pool: CpuPool, repo_factory: F) -> Self {
        Self {
            db_pool,
            cpu_pool,
            repo_factory,
        }
    }
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > UserRolesService for UserRolesServiceImpl<T, M, F>
{
    /// Returns role by user ID
    fn get_roles(&self, user_id: UserId) -> ServiceFuture<Vec<UsersRole>> {
        let db_pool = self.db_pool.clone();
        let repo_factory = self.repo_factory.clone();

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(Error::Connection).into())
                        .and_then(move |conn| {
                            let user_roles_repo = repo_factory.create_user_roles_repo(&*conn);
                            user_roles_repo.list_for_user(user_id)
                        })
                })
                .map_err(|e| e.context("Service UserRoles, get_roles endpoint error occured.").into()),
        )
    }

    /// Deletes specific user role
    fn delete(&self, payload: OldUserRole) -> ServiceFuture<UserRole> {
        let db_pool = self.db_pool.clone();
        let repo_factory = self.repo_factory.clone();

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(Error::Connection).into())
                        .and_then(move |conn| {
                            let user_roles_repo = repo_factory.create_user_roles_repo(&*conn);
                            user_roles_repo.delete(payload)
                        })
                })
                .map_err(|e| e.context("Service UserRoles, delete endpoint error occured.").into()),
        )
    }

    /// Creates new user_role
    fn create(&self, new_user_role: NewUserRole) -> ServiceFuture<UserRole> {
        let db_pool = self.db_pool.clone();
        let repo_factory = self.repo_factory.clone();

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(Error::Connection).into())
                        .and_then(move |conn| {
                            let user_roles_repo = repo_factory.create_user_roles_repo(&*conn);
                            user_roles_repo.create(new_user_role)
                        })
                })
                .map_err(|e| e.context("Service UserRoles, create endpoint error occured.").into()),
        )
    }

    /// Deletes default roles for user
    fn delete_default(&self, user_id_arg: UserId) -> ServiceFuture<UserRole> {
        let db_pool = self.db_pool.clone();
        let repo_factory = self.repo_factory.clone();

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(Error::Connection).into())
                        .and_then(move |conn| {
                            let user_roles_repo = repo_factory.create_user_roles_repo(&*conn);
                            user_roles_repo.delete_by_user_id(user_id_arg)
                        })
                })
                .map_err(|e| e.context("Service UserRoles, delete_default endpoint error occured.").into()),
        )
    }

    /// Creates default roles for user
    fn create_default(&self, user_id_arg: UserId) -> ServiceFuture<UserRole> {
        let db_pool = self.db_pool.clone();
        let repo_factory = self.repo_factory.clone();

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(Error::Connection).into())
                        .and_then(move |conn| {
                            let defaul_role = NewUserRole {
                                user_id: user_id_arg,
                                role: UsersRole::User,
                            };
                            let user_roles_repo = repo_factory.create_user_roles_repo(&*conn);
                            user_roles_repo.create(defaul_role)
                        })
                })
                .map_err(|e| e.context("Service UserRoles, create_default endpoint error occured.").into()),
        )
    }
}

#[cfg(test)]
pub mod tests {
    use futures_cpupool::CpuPool;
    use r2d2;
    use tokio_core::reactor::Core;

    use stq_types::*;

    use super::*;
    use models::*;
    use repos::repo_factory::tests::*;

    fn create_user_roles_service() -> UserRolesServiceImpl<MockConnection, MockConnectionManager, ReposFactoryMock> {
        let manager = MockConnectionManager::default();
        let db_pool = r2d2::Pool::builder().build(manager).expect("Failed to create connection pool");
        let cpu_pool = CpuPool::new(1);

        UserRolesServiceImpl {
            db_pool: db_pool,
            cpu_pool: cpu_pool,
            repo_factory: MOCK_REPO_FACTORY,
        }
    }

    pub fn create_new_user_roles(user_id: UserId) -> NewUserRole {
        NewUserRole {
            user_id: user_id,
            role: UsersRole::User,
        }
    }

    pub fn delete_user_roles(user_id: UserId) -> OldUserRole {
        OldUserRole {
            user_id: user_id,
            role: UsersRole::User,
        }
    }

    #[test]
    fn test_get_user_roles() {
        let mut core = Core::new().unwrap();
        let service = create_user_roles_service();
        let work = service.get_roles(UserId(1));
        let result = core.run(work).unwrap();
        assert_eq!(result[0], UsersRole::Superuser);
    }

    #[test]
    fn test_create_user_roles() {
        let mut core = Core::new().unwrap();
        let service = create_user_roles_service();
        let new_user_roles = create_new_user_roles(MOCK_USER_ID);
        let work = service.create(new_user_roles);
        let result = core.run(work).unwrap();
        assert_eq!(result.user_id, UserId(1));
    }

}
