use failure::Error as FailureError;
use failure::Fail;
use serde_json;

use stq_types::{BaseProductId, ProductPrice, StoreId};

use errors::Error;
use schema::local_shipping;
use stq_static_resources::DeliveryCompany;

#[derive(Serialize, Queryable, Insertable, Debug)]
#[table_name = "local_shipping"]
pub struct LocalShippingRaw {
    pub id: i32,
    pub base_product_id: BaseProductId,
    pub pickup: bool,
    pub country: String,
    pub companies: serde_json::Value,
    pub store_id: StoreId,
    pub pickup_price: Option<f64>,
}

#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "local_shipping"]
pub struct NewLocalShippingRaw {
    pub base_product_id: BaseProductId,
    pub store_id: StoreId,
    pub pickup: bool,
    pub country: String,
    pub pickup_price: Option<f64>,
    pub companies: serde_json::Value,
}

#[derive(Serialize, Deserialize, Insertable, AsChangeset, Clone, Debug)]
#[table_name = "local_shipping"]
pub struct UpdateLocalShippingRaw {
    pub country: Option<String>,
    pub pickup: Option<bool>,
    pub pickup_price: Option<f64>,
    pub companies: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LocalShippingCompany {
    pub company: DeliveryCompany,
    pub price: Option<ProductPrice>,
    pub duration_days: Option<i32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LocalShipping {
    pub id: i32,
    pub base_product_id: BaseProductId,
    pub store_id: StoreId,
    pub pickup: bool,
    pub pickup_price: Option<f64>,
    pub companies: Vec<LocalShippingCompany>,
}

impl LocalShipping {
    pub fn from_raw(shipping: LocalShippingRaw) -> Result<Self, FailureError> {
        let companies =
            serde_json::from_value(shipping.companies).map_err(|e| e.context("Can not parse companies from db").context(Error::Parse))?;
        Ok(Self {
            id: shipping.id,
            base_product_id: shipping.base_product_id,
            store_id: shipping.store_id,
            pickup: shipping.pickup,
            pickup_price: shipping.pickup_price,
            companies,
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewLocalShipping {
    pub base_product_id: BaseProductId,
    pub store_id: StoreId,
    pub pickup: bool,
    pub country: String,
    pub pickup_price: Option<f64>,
    pub companies: Vec<LocalShippingCompany>,
}

impl NewLocalShipping {
    pub fn to_raw(self) -> Result<NewLocalShippingRaw, FailureError> {
        let companies =
            serde_json::to_value(self.companies).map_err(|e| e.context("Can not parse companies from value").context(Error::Parse))?;
        Ok(NewLocalShippingRaw {
            base_product_id: self.base_product_id,
            store_id: self.store_id,
            pickup: self.pickup,
            pickup_price: self.pickup_price,
            country: self.country,
            companies,
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UpdateLocalShipping {
    pub pickup: Option<bool>,
    pub pickup_price: Option<f64>,
    pub country: Option<String>,
    pub companies: Option<Vec<LocalShippingCompany>>,
}

impl UpdateLocalShipping {
    pub fn to_raw(self) -> Result<UpdateLocalShippingRaw, FailureError> {
        let companies = match self.companies {
            Some(v) => serde_json::to_value(v)
                .map(Some)
                .map_err(|e| e.context("Can not parse companies from value").context(Error::Parse))?,
            None => None,
        };

        Ok(UpdateLocalShippingRaw {
            companies,
            pickup: self.pickup,
            pickup_price: self.pickup_price,
            country: self.country,
        })
    }
}
