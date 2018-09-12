use stq_types::{CompanyPackageId, ProductPrice};

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

#[derive(Serialize, Deserialize, Debug)]
pub struct AvailablePackageForUser {
    pub id: CompanyPackageId,
    pub name: String,
    pub logo: String,
    pub price: Option<ProductPrice>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AvailableShipppingForUser {
    pub packages: Vec<AvailablePackageForUser>,
    pub pickups: Option<Pickups>,
}
