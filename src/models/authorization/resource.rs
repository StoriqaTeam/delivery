//! Enum for resources available in ACLs
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Resource {
    Companies,
    CompaniesPackages,
    Countries,
    Packages,
    Pickups,
    Products,
    UserAddresses,
    UserRoles,
}

impl fmt::Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Resource::Companies => write!(f, "companies"),
            Resource::CompaniesPackages => write!(f, "companies_packages"),
            Resource::Countries => write!(f, "countries"),
            Resource::Packages => write!(f, "packages"),
            Resource::Pickups => write!(f, "pickups"),
            Resource::Products => write!(f, "products"),
            Resource::UserAddresses => write!(f, "user addresses"),
            Resource::UserRoles => write!(f, "user roles"),
        }
    }
}
