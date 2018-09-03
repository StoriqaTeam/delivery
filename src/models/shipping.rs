use models::{Country, NewPickups, NewProducts, Pickups, Products};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Shipping {
    pub items: Vec<ShippingProducts>,
    pub pickup: Option<Pickups>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewShipping {
    pub items: Vec<NewProducts>,
    pub pickup: Option<NewPickups>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ShippingProducts {
    pub product: Products,
    pub deliveries_to: Vec<Country>,
}
