//! Models for managing Roles
use stq_types::{UserId, UsersRole};
use std::time::SystemTime;

use schema::user_roles;

#[derive(Serialize, Queryable, Insertable, Debug)]
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
