//! Models for managing Roles

use serde_json;

use stq_types::{DeliveryRole, RoleId, UserId};

use schema::roles;

#[derive(Serialize, Deserialize, Queryable, Insertable, Debug)]
#[table_name = "roles"]
pub struct UserRole {
    pub id: RoleId,
    pub user_id: UserId,
    pub name: DeliveryRole,
    pub data: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Insertable)]
#[table_name = "roles"]
pub struct NewUserRole {
    pub id: RoleId,
    pub user_id: UserId,
    pub name: DeliveryRole,
    pub data: Option<serde_json::Value>,
}
