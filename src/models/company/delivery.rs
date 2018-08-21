use schema::delivery_from;
use schema::delivery_to;

use super::DeliveryCompany;

use serde_json;

#[derive(Serialize, Queryable, Insertable, Debug)]
#[table_name = "delivery_from"]
pub struct DeliveryFrom {
    pub id: i32,
    pub company_id: DeliveryCompany,
    pub country: String,
    pub restriction_name: String,
}

#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "delivery_from"]
pub struct NewDeliveryFrom {
    pub company_id: DeliveryCompany,
    pub country: String,
    pub restriction_name: String,
}

#[derive(Serialize, Deserialize, Insertable, AsChangeset, Clone, Debug)]
#[table_name = "delivery_from"]
pub struct UpdateDeliveryFrom {
    pub company_id: DeliveryCompany,
    pub country: String,
    pub restriction_name: String,
}

#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "delivery_from"]
pub struct OldDeliveryFrom {
    pub company_id: DeliveryCompany,
    pub country: String,
    pub restriction_name: String,
}

#[derive(Serialize, Queryable, Insertable, Debug)]
#[table_name = "delivery_to"]
pub struct DeliveryTo {
    pub id: i32,
    pub company_id: DeliveryCompany,
    pub country: String,
    pub additional_info: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "delivery_to"]
pub struct NewDeliveryTo {
    pub company_id: DeliveryCompany,
    pub country: String,
    pub additional_info: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Insertable, AsChangeset, Clone, Debug)]
#[table_name = "delivery_to"]
pub struct UpdateDeliveryTo {
    pub company_id: DeliveryCompany,
    pub country: String,
    pub additional_info: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "delivery_to"]
pub struct OldDeliveryTo {
    pub company_id: DeliveryCompany,
    pub country: String,
}
