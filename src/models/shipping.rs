use models::{NewPickups, NewProducts, Pickups, Products};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Shipping {
    pub items: Vec<Products>,
    pub pickup: Option<Pickups>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewShipping {
    pub items: Vec<NewProducts>,
    pub pickup: Option<NewPickups>,
}
