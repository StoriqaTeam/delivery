//! Models for managing Roles
use std::time::SystemTime;
use stq_types::{UserId, UsersRole};

use schema::user_roles;

#[derive(Serialize, Deserialize, Queryable, Insertable, Debug)]
#[table_name = "user_roles"]
pub struct UserRole {
    pub id: i32,
    pub user_id: UserId,
    pub role: UsersRole,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "user_roles"]
pub struct NewUserRole {
    pub user_id: UserId,
    pub role: UsersRole,
}

#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "user_roles"]
pub struct OldUserRole {
    pub user_id: UserId,
    pub role: UsersRole,
}
