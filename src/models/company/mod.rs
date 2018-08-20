pub mod delivery;
pub mod restriction;

pub use self::delivery::*;
pub use self::restriction::*;

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, Debug, DieselTypes)]
pub enum DeliveryCompany {
    UPS,
}
