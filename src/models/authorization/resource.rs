//! Enum for resources available in ACLs
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Resource {
    UserRoles,
    Restrictions,
    DeliveryFrom,
    DeliveryTo,
}

impl fmt::Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Resource::UserRoles => write!(f, "user roles"),
            Resource::Restrictions => write!(f, "restrictions"),
            Resource::DeliveryFrom => write!(f, "delivery_from"),
            Resource::DeliveryTo => write!(f, "delivery_to"),
        }
    }
}
