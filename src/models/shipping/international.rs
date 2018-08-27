use failure::Error as FailureError;
use failure::Fail;
use serde_json;

use stq_types::{BaseProductId, ProductPrice, StoreId};

use errors::Error;
use schema::international_shipping;
use stq_static_resources::DeliveryCompany;

#[derive(Serialize, Queryable, Insertable, Debug)]
#[table_name = "international_shipping"]
pub struct InternationalShippingRaw {
    pub id: i32,
    pub base_product_id: BaseProductId,
    pub companies: serde_json::Value,
    pub store_id: StoreId,
}

#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "international_shipping"]
pub struct NewInternationalShippingRaw {
    pub base_product_id: BaseProductId,
    pub store_id: StoreId,
    pub companies: serde_json::Value,
}

#[derive(Serialize, Deserialize, Insertable, AsChangeset, Clone, Debug)]
#[table_name = "international_shipping"]
pub struct UpdateInternationalShippingRaw {
    pub companies: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InternationalShippingCompany {
    pub company: DeliveryCompany,
    pub price: Option<ProductPrice>,
    pub countries: Vec<String>,
    pub duration_days: Option<i32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InternationalShipping {
    pub id: i32,
    pub base_product_id: BaseProductId,
    pub store_id: StoreId,
    pub companies: Vec<InternationalShippingCompany>,
}

impl InternationalShipping {
    pub fn from_raw(shipping: InternationalShippingRaw) -> Result<Self, FailureError> {
        let companies =
            serde_json::from_value(shipping.companies).map_err(|e| e.context("Can not parse companies from db").context(Error::Parse))?;
        Ok(Self {
            id: shipping.id,
            base_product_id: shipping.base_product_id,
            store_id: shipping.store_id,
            companies,
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewInternationalShipping {
    pub base_product_id: BaseProductId,
    pub store_id: StoreId,
    pub companies: Vec<InternationalShippingCompany>,
}

impl NewInternationalShipping {
    pub fn to_raw(self) -> Result<NewInternationalShippingRaw, FailureError> {
        let companies =
            serde_json::to_value(self.companies).map_err(|e| e.context("Can not parse companies from value").context(Error::Parse))?;
        Ok(NewInternationalShippingRaw {
            base_product_id: self.base_product_id,
            store_id: self.store_id,
            companies,
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UpdateInternationalShipping {
    pub companies: Option<Vec<InternationalShippingCompany>>,
}

impl UpdateInternationalShipping {
    pub fn to_raw(self) -> Result<UpdateInternationalShippingRaw, FailureError> {
        let companies = match self.companies {
            Some(v) => serde_json::to_value(v)
                .map(Some)
                .map_err(|e| e.context("Can not parse companies from value").context(Error::Parse))?,
            None => None,
        };
        Ok(UpdateInternationalShippingRaw { companies })
    }
}
