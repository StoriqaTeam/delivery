use failure::Error as FailureError;
use failure::Fail;
use serde_json;

use stq_types::{Alpha3, BaseProductId, CompanyPackageId, ProductPrice, StoreId};

use errors::Error;
use schema::products;

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, DieselTypes)]
pub enum ShippingVariant {
    Local,
    International,
}

#[derive(Serialize, Queryable, Insertable, Debug, QueryableByName)]
#[table_name = "products"]
pub struct ProductsRaw {
    pub id: i32,
    pub base_product_id: BaseProductId,
    pub store_id: StoreId,
    pub company_package_id: CompanyPackageId,
    pub price: Option<ProductPrice>,
    pub deliveries_to: serde_json::Value,
    pub shipping: ShippingVariant,
}

#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "products"]
pub struct NewProductsRaw {
    pub base_product_id: BaseProductId,
    pub store_id: StoreId,
    pub company_package_id: CompanyPackageId,
    pub price: Option<ProductPrice>,
    pub deliveries_to: serde_json::Value,
    pub shipping: ShippingVariant,
}

#[derive(Serialize, Deserialize, Insertable, AsChangeset, Clone, Debug)]
#[table_name = "products"]
pub struct UpdateProductsRaw {
    pub price: Option<ProductPrice>,
    pub deliveries_to: Option<serde_json::Value>,
    pub shipping: Option<ShippingVariant>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Products {
    pub id: i32,
    pub base_product_id: BaseProductId,
    pub store_id: StoreId,
    pub company_package_id: CompanyPackageId,
    pub price: Option<ProductPrice>,
    pub deliveries_to: Vec<Alpha3>,
    pub shipping: ShippingVariant,
}

impl ProductsRaw {
    pub fn to_products(self) -> Result<Products, FailureError> {
        let deliveries_to =
            serde_json::from_value(self.deliveries_to).map_err(|e| e.context("Can not parse products from db").context(Error::Parse))?;
        Ok(Products {
            id: self.id,
            base_product_id: self.base_product_id,
            store_id: self.store_id,
            company_package_id: self.company_package_id,
            price: self.price,
            deliveries_to,
            shipping: self.shipping,
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewProducts {
    pub base_product_id: BaseProductId,
    pub store_id: StoreId,
    pub company_package_id: CompanyPackageId,
    pub price: Option<ProductPrice>,
    pub deliveries_to: Vec<Alpha3>,
    pub shipping: ShippingVariant,
}

impl NewProducts {
    pub fn to_raw(self) -> Result<NewProductsRaw, FailureError> {
        let deliveries_to =
            serde_json::to_value(self.deliveries_to).map_err(|e| e.context("Can not parse products from db").context(Error::Parse))?;
        Ok(NewProductsRaw {
            base_product_id: self.base_product_id,
            store_id: self.store_id,
            company_package_id: self.company_package_id,
            price: self.price,
            deliveries_to,
            shipping: self.shipping,
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UpdateProducts {
    pub price: Option<ProductPrice>,
    pub deliveries_to: Option<Vec<Alpha3>>,
    pub shipping: Option<ShippingVariant>,
}

impl UpdateProducts {
    pub fn to_raw(self) -> Result<UpdateProductsRaw, FailureError> {
        let deliveries_to = match self.deliveries_to {
            Some(v) => serde_json::to_value(v)
                .map(Some)
                .map_err(|e| e.context("Can not parse products from value").context(Error::Parse))?,
            None => None,
        };

        Ok(UpdateProductsRaw {
            price: self.price,
            deliveries_to,
            shipping: self.shipping,
        })
    }
}
