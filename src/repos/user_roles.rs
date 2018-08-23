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

use stq_types::{RoleId, StoresRole, UserId};

use models::authorization::*;
use models::{NewUserRole, UserRole};
use repos::legacy_acl::*;
use repos::types::RepoResult;
use repos::RolesCacheImpl;
use schema::roles::dsl::*;

/// UserRoles repository for handling UserRoles
pub trait UserRolesRepo {
    /// Returns list of user_roles for a specific user
    fn list_for_user(&self, user_id: UserId) -> RepoResult<Vec<StoresRole>>;

    /// Create a new user role
    fn create(&self, payload: NewUserRole) -> RepoResult<UserRole>;

    /// Delete roles of a user
    fn delete_by_user_id(&self, user_id: UserId) -> RepoResult<Vec<UserRole>>;

    /// Delete user roles by id
    fn delete_by_id(&self, id: RoleId) -> RepoResult<UserRole>;
}

/// Implementation of UserRoles trait
pub struct UserRolesRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<Acl<Resource, Action, Scope, FailureError, UserRole>>,
    pub cached_roles: RolesCacheImpl,
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> UserRolesRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<Acl<Resource, Action, Scope, FailureError, UserRole>>, cached_roles: RolesCacheImpl) -> Self {
        Self {
            db_conn,
            acl,
            cached_roles,
        }
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> UserRolesRepo for UserRolesRepoImpl<'a, T> {
    /// Returns list of user_roles for a specific user
    fn list_for_user(&self, user_id_value: UserId) -> RepoResult<Vec<StoresRole>> {
        debug!("list user roles for id {}.", user_id_value);
        if self.cached_roles.contains(user_id_value) {
            let user_roles = self.cached_roles.get(user_id_value);
            Ok(user_roles)
        } else {
            let query = roles.filter(user_id.eq(user_id_value));
            query
                .get_results::<UserRole>(self.db_conn)
                .and_then(|user_roles_arg| {
                    let user_roles = user_roles_arg
                        .into_iter()
                        .map(|user_role| user_role.name)
                        .collect::<Vec<StoresRole>>();
                    Ok(user_roles)
                })
                .and_then(|user_roles| {
                    if !user_roles.is_empty() {
                        self.cached_roles.add_roles(user_id_value, &user_roles);
                    }
                    Ok(user_roles)
                })
                .map_err(|e| {
                    e.context(format!("List user roles for user {} error occured.", user_id_value))
                        .into()
                })
        }
    }

    /// Create a new user role
    fn create(&self, payload: NewUserRole) -> RepoResult<UserRole> {
        debug!("create new user role {:?}.", payload);
        self.cached_roles.remove(payload.user_id);
        let query = diesel::insert_into(roles).values(&payload);
        query
            .get_result(self.db_conn)
            .map_err(|e| e.context(format!("Create a new user role {:?} error occured", payload)).into())
    }

    /// Delete roles of a user
    fn delete_by_user_id(&self, user_id_arg: UserId) -> RepoResult<Vec<UserRole>> {
        debug!("delete user {} role.", user_id_arg);
        self.cached_roles.remove(user_id_arg);
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
                self.cached_roles.remove(user_role.user_id);
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
