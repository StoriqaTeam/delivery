use models::packages::DeliveriesTo;
use schema::companies_packages;

#[derive(Serialize, Deserialize, Associations, Queryable, Debug)]
#[table_name = "companies_packages"]
pub struct CompaniesPackages {
    pub id: i32,
    pub company_id: i32,
    pub package_id: i32,
}

#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "companies_packages"]
pub struct NewCompaniesPackages {
    pub company_id: i32,
    pub package_id: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AvailablePackages {
    pub id: i32,
    pub name: String,
    pub deliveries_to: Vec<DeliveriesTo>,
}
