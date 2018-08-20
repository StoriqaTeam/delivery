use schema::company_restrictions;

#[derive(Serialize, Queryable, Insertable, Debug)]
#[table_name = "company_restrictions"]
pub struct CompanyRestriction {
    pub id: i32,
    pub name: String,
    pub max_weight: f64,
    pub max_size: f64,
}

#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "company_restrictions"]
pub struct NewCompanyRestriction {
    pub name: String,
    pub max_weight: f64,
    pub max_size: f64,
}

#[derive(Serialize, Deserialize, Insertable, AsChangeset, Clone, Debug)]
#[table_name = "company_restrictions"]
pub struct UpdateCompanyRestriction {
    pub name: String,
    pub max_weight: f64,
    pub max_size: f64,
}

#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "company_restrictions"]
pub struct OldCompanyRestriction {
    pub name: String,
}
