use stq_static_resources::DeliveryCompany;
use schema::delivery_from;

#[derive(Serialize, Deserialize, Queryable, Insertable, Debug)]
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
