use schema::restrictions;

#[derive(Serialize, Deserialize, Queryable, Insertable, Debug)]
#[table_name = "restrictions"]
pub struct Restriction {
    pub id: i32,
    pub name: String,
    pub max_weight: f64,
    pub max_size: f64,
}

#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "restrictions"]
pub struct NewRestriction {
    pub name: String,
    pub max_weight: f64,
    pub max_size: f64,
}

#[derive(Serialize, Deserialize, Insertable, AsChangeset, Clone, Debug)]
#[table_name = "restrictions"]
pub struct UpdateRestriction {
    pub name: String,
    pub max_weight: Option<f64>,
    pub max_size: Option<f64>,
}
