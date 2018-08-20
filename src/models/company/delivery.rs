use schema::company_delivery_from;
use schema::company_delivery_to;

#[derive(Serialize, Queryable, Insertable, Debug)]
#[table_name = "company_delivery_from"]
pub struct CompanyDeliveryFrom {
    pub id: i32,
    pub company_id: String,
    pub country: String,
    pub company_restriction: String,
}

#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "company_delivery_from"]
pub struct NewCompanyDeliveryFrom {
    pub company_id: String,
    pub country: String,
    pub company_restriction: String,
}

#[derive(Serialize, Deserialize, Insertable, AsChangeset, Clone, Debug)]
#[table_name = "company_delivery_from"]
pub struct UpdateCompanyDeliveryFrom {
    pub company_id: String,
    pub country: String,
    pub company_restriction: String,
}

#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "company_delivery_from"]
pub struct OldCompanyDeliveryFrom {
    pub company_id: String,
    pub country: String,
    pub company_restriction: String,
}

#[derive(Serialize, Queryable, Insertable, Debug)]
#[table_name = "company_delivery_to"]
pub struct CompanyDeliveryTo {
    pub id: i32,
    pub company_id: String,
    pub country: String,
}

#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "company_delivery_to"]
pub struct NewCompanyDeliveryTo {
    pub company_id: String,
    pub country: String,
}

#[derive(Serialize, Deserialize, Insertable, AsChangeset, Clone, Debug)]
#[table_name = "company_delivery_to"]
pub struct UpdateCompanyDeliveryTo {
    pub company_id: String,
    pub country: String,
}

#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "company_delivery_to"]
pub struct OldCompanyDeliveryTo {
    pub company_id: String,
    pub country: String,
}