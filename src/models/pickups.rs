use schema::pickups;
use stq_types::{BaseProductId, ProductPrice, StoreId};

#[derive(Serialize, Deserialize, Associations, Queryable, Debug)]
#[table_name = "pickups"]
pub struct Pickups {
    pub id: i32,
    pub base_product_id: BaseProductId,
    pub store_id: StoreId,
    pub pickup: bool,
    pub price: Option<ProductPrice>,
}

#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "pickups"]
pub struct NewPickups {
    pub base_product_id: BaseProductId,
    pub store_id: StoreId,
    pub pickup: bool,
    pub price: Option<ProductPrice>,
}

#[derive(Serialize, Deserialize, Insertable, AsChangeset, Clone, Debug)]
#[table_name = "pickups"]
pub struct UpdatePickups {
    pub pickup: Option<bool>,
    pub price: Option<ProductPrice>,
}
