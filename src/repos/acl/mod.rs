//! Repos is a module responsible for interacting with access control lists
//! Authorization module contains authorization logic for the repo layer app

#[macro_use]
pub mod macros;
pub mod legacy_acl;
pub mod roles_cache;

pub use self::roles_cache::RolesCacheImpl;

use std::collections::HashMap;
use std::rc::Rc;

use errors::Error;
use failure::Error as FailureError;

use stq_types::{DeliveryRole, UserId};

use self::legacy_acl::{Acl, CheckScope};

use models::authorization::*;

pub fn check<T>(
    acl: &Acl<Resource, Action, Scope, FailureError, T>,
    resource: Resource,
    action: Action,
    scope_checker: &CheckScope<Scope, T>,
    obj: Option<&T>,
) -> Result<(), FailureError> {
    acl.allows(resource, action, scope_checker, obj).and_then(|allowed| {
        if allowed {
            Ok(())
        } else {
            Err(format_err!("Denied request to do {:?} on {:?}", action, resource)
                .context(Error::Forbidden)
                .into())
        }
    })
}

/// ApplicationAcl contains main logic for manipulation with resources
#[derive(Clone)]
pub struct ApplicationAcl {
    acls: Rc<HashMap<DeliveryRole, Vec<Permission>>>,
    roles: Vec<DeliveryRole>,
    user_id: UserId,
}

impl ApplicationAcl {
    pub fn new(roles: Vec<DeliveryRole>, user_id: UserId) -> Self {
        let mut hash = ::std::collections::HashMap::new();

        hash.insert(
            DeliveryRole::Superuser,
            vec![
                permission!(Resource::Companies),
                permission!(Resource::CompaniesPackages),
                permission!(Resource::Countries),
                permission!(Resource::Packages),
                permission!(Resource::Pickups),
                permission!(Resource::Products),
                permission!(Resource::UserAddresses),
                permission!(Resource::UserRoles),
            ],
        );

        hash.insert(
            DeliveryRole::User,
            vec![
                permission!(Resource::Companies, Action::Read),
                permission!(Resource::CompaniesPackages, Action::Read),
                permission!(Resource::Countries, Action::Read),
                permission!(Resource::Packages, Action::Read),
                permission!(Resource::Pickups, Action::Read),
                permission!(Resource::Products, Action::Read),
                permission!(Resource::UserAddresses, Action::All, Scope::Owned),
                permission!(Resource::UserRoles, Action::Read, Scope::Owned),
            ],
        );

        hash.insert(
            DeliveryRole::StoreManager,
            vec![
                permission!(Resource::Pickups, Action::All, Scope::Owned),
                permission!(Resource::Products, Action::All, Scope::Owned),
            ],
        );

        ApplicationAcl {
            acls: Rc::new(hash),
            roles,
            user_id,
        }
    }
}
impl<T> Acl<Resource, Action, Scope, FailureError, T> for ApplicationAcl {
    fn allows(
        &self,
        resource: Resource,
        action: Action,
        scope_checker: &CheckScope<Scope, T>,
        obj: Option<&T>,
    ) -> Result<bool, FailureError> {
        let empty: Vec<Permission> = Vec::new();
        let user_id = &self.user_id;
        let hashed_acls = self.acls.clone();
        let acls = self
            .roles
            .iter()
            .flat_map(|role| hashed_acls.get(role).unwrap_or(&empty))
            .filter(|permission| (permission.resource == resource) && ((permission.action == action) || (permission.action == Action::All)))
            .filter(|permission| scope_checker.is_in_scope(*user_id, &permission.scope, obj));
        if acls.count() > 0 {
            Ok(true)
        } else {
            error!("Denied request from user {} to do {} on {}.", user_id, action, resource);
            Ok(false)
        }
    }
}

/// UnauthorizedAcl contains main logic for manipulation with resources
#[derive(Clone, Default)]
pub struct UnauthorizedAcl;

impl<T> Acl<Resource, Action, Scope, FailureError, T> for UnauthorizedAcl {
    fn allows(
        &self,
        resource: Resource,
        action: Action,
        _scope_checker: &CheckScope<Scope, T>,
        _obj: Option<&T>,
    ) -> Result<bool, FailureError> {
        if action == Action::Read {
            match resource {
                Resource::Companies => Ok(true),
                Resource::CompaniesPackages => Ok(true),
                Resource::Countries => Ok(true),
                Resource::Packages => Ok(true),
                Resource::Pickups => Ok(true),
                Resource::Products => Ok(true),
                _ => Ok(false),
            }
        } else {
            error!("Denied unauthorized request to do {} on {}.", action, resource);
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    // write tests
}
