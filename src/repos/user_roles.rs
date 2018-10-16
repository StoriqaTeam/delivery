//! Repo for user_roles table. UserRole is an entity that connects
//! users and roles. I.e. this table is for user has-many roles
//! relationship

use diesel;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::Connection;
use failure::Error as FailureError;
use failure::Fail;

use stq_types::{DeliveryRole, RoleId, UserId};

use models::authorization::*;
use models::{NewUserRole, UserRole};
use repos::legacy_acl::*;
use repos::types::RepoResult;
use repos::RolesCacheImpl;
use schema::roles::dsl::*;

/// UserRoles repository for handling UserRoles
pub trait UserRolesRepo {
    /// Returns list of user_roles for a specific user
    fn list_for_user(&self, user_id: UserId) -> RepoResult<Vec<DeliveryRole>>;

    /// Create a new user role
    fn create(&self, payload: NewUserRole) -> RepoResult<UserRole>;

    /// Delete roles of a user
    fn delete_by_user_id(&self, user_id: UserId) -> RepoResult<Vec<UserRole>>;

    /// Delete user roles by id
    fn delete_by_id(&self, id: RoleId) -> RepoResult<UserRole>;
}

/// Implementation of UserRoles trait
pub struct UserRolesRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub acl: Box<Acl<Resource, Action, Scope, FailureError, UserRole>>,
    pub db_conn: &'a T,
    pub roles_cache: RolesCacheImpl,
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> UserRolesRepoImpl<'a, T> {
    pub fn new(
        db_conn: &'a T, acl: Box<Acl<Resource, Action, Scope, FailureError, UserRole>>,
        roles_cache: RolesCacheImpl,
    ) -> Self {
        Self {
            acl,
            db_conn,
            roles_cache,
        }
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> UserRolesRepo for UserRolesRepoImpl<'a, T> {
    /// Returns list of user_roles for a specific user
    fn list_for_user(&self, user_id_value: UserId) -> RepoResult<Vec<DeliveryRole>> {
        debug!("list user roles for id {}.", user_id_value);

        if let Some(user_roles) = self.roles_cache.get(user_id_value) {
            Ok(user_roles)
        } else {
            let query = roles.filter(user_id.eq(user_id_value));
            query.get_results::<UserRole>(self.db_conn)
                .map(|user_roles_arg| {
                    let user_roles = user_roles_arg
                        .into_iter()
                        .map(|user_role| user_role.name)
                        .collect::<Vec<DeliveryRole>>();
                        
                    if !user_roles.is_empty() {
                        self.roles_cache.set(user_id_value, &user_roles);
                    }

                    user_roles
                })
                .map_err(|e| e.context(format!("List user roles for user {} error occured.", user_id_value)).into())
        }
    }

    /// Create a new user role
    fn create(&self, payload: NewUserRole) -> RepoResult<UserRole> {
        debug!("create new user role {:?}.", payload);
        self.roles_cache.remove(payload.user_id);
        let query = diesel::insert_into(roles).values(&payload);
        query
            .get_result(self.db_conn)
            .map_err(|e| e.context(format!("Create a new user role {:?} error occured", payload)).into())
    }

    /// Delete roles of a user
    fn delete_by_user_id(&self, user_id_arg: UserId) -> RepoResult<Vec<UserRole>> {
        debug!("delete user {} role.", user_id_arg);
        self.roles_cache.remove(user_id_arg);
        let filtered = roles.filter(user_id.eq(user_id_arg));
        let query = diesel::delete(filtered);
        query
            .get_results(self.db_conn)
            .map_err(|e| e.context(format!("Delete user {} roles error occured", user_id_arg)).into())
    }

    /// Delete user roles by id
    fn delete_by_id(&self, id_arg: RoleId) -> RepoResult<UserRole> {
        debug!("delete user role by id {}.", id_arg);
        let filtered = roles.filter(id.eq(id_arg));
        let query = diesel::delete(filtered);
        query
            .get_result(self.db_conn)
            .map_err(|e| e.context(format!("Delete role {} error occured", id_arg)).into())
            .map(|user_role: UserRole| {
                self.roles_cache.remove(user_role.user_id);
                user_role
            })
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, UserRole>
    for UserRolesRepoImpl<'a, T>
{
    fn is_in_scope(&self, user_id_arg: UserId, scope: &Scope, obj: Option<&UserRole>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => {
                if let Some(user_role) = obj {
                    user_role.user_id == user_id_arg
                } else {
                    false
                }
            }
        }
    }
}
