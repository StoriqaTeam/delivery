//! Models for managing user delivery address
use std::time::SystemTime;

use validator::Validate;

use stq_types::UserId;

use schema::user_addresses;

#[derive(Serialize, Queryable, Insertable, Debug, Deserialize)]
#[table_name = "user_addresses"]
pub struct UserAddress {
    pub id: i32,
    pub user_id: UserId,
    pub administrative_area_level_1: Option<String>,
    pub administrative_area_level_2: Option<String>,
    pub country: String,
    pub locality: Option<String>,
    pub political: Option<String>,
    pub postal_code: String,
    pub route: Option<String>,
    pub street_number: Option<String>,
    pub address: Option<String>,
    pub is_priority: bool,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
    pub country_code: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Insertable, Validate)]
#[table_name = "user_addresses"]
pub struct NewUserAddress {
    pub user_id: UserId,
    pub administrative_area_level_1: Option<String>,
    pub administrative_area_level_2: Option<String>,
    #[validate(length(min = "1", message = "Country must not be empty"))]
    pub country: String,
    pub locality: Option<String>,
    pub political: Option<String>,
    #[validate(length(min = "1", message = "Postal code must not be empty"))]
    pub postal_code: String,
    pub route: Option<String>,
    pub street_number: Option<String>,
    pub address: Option<String>,
    pub is_priority: bool,
    #[validate(length(min = "1", message = "Country code must not be empty"))]
    pub country_code: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Insertable, AsChangeset, Validate)]
#[table_name = "user_addresses"]
pub struct UpdateUserAddress {
    pub administrative_area_level_1: Option<String>,
    pub administrative_area_level_2: Option<String>,
    #[validate(length(min = "1", message = "Country must not be empty"))]
    pub country: Option<String>,
    pub locality: Option<String>,
    pub political: Option<String>,
    #[validate(length(min = "1", message = "Postal code must not be empty"))]
    pub postal_code: Option<String>,
    pub route: Option<String>,
    pub street_number: Option<String>,
    pub address: Option<String>,
    pub is_priority: Option<bool>,
    #[validate(length(min = "1", message = "Country code must not be empty"))]
    pub country_code: Option<String>,
}
