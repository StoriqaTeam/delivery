use failure::Error as FailureError;
use failure::Fail;
use serde_json;
use validator::{Validate, ValidationError, ValidationErrors};

use stq_types::{Alpha3, BaseProductId, CompanyPackageId, ProductPrice, ShippingId, StoreId};

use errors::Error;
use models::{get_country_from_forest, Company, Packages, ShipmentMeasurements, ShippingRate};
use schema::products;

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, DieselTypes)]
pub enum ShippingVariant {
    Local,
    International,
}

#[derive(Serialize, Queryable, Insertable, Debug, QueryableByName)]
#[table_name = "products"]
pub struct ProductsRaw {
    pub id: ShippingId,
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
    pub id: ShippingId,
    pub base_product_id: BaseProductId,
    pub store_id: StoreId,
    pub company_package_id: CompanyPackageId,
    pub price: Option<ProductPrice>,
    pub deliveries_to: Vec<Alpha3>,
    pub shipping: ShippingVariant,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ShippingPrice {
    Fixed {
        price: f64,
    },
    Calculated {
        measurements: ShipmentMeasurements,
        shipping_rate: ShippingRate,
        billable_weight_g: u32,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProductWithShippingPrice {
    pub product: Products,
    pub shipping_price: ShippingPrice,
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

    pub fn get_deliveries_to(&self) -> Result<Vec<Alpha3>, FailureError> {
        let used_codes = serde_json::from_value(self.deliveries_to.clone())
            .map_err(|e| e.context("Can not parse deliveries_to from db").context(Error::Parse))?;

        Ok(used_codes)
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
    pub measurements: Option<ShipmentMeasurements>,
    pub delivery_from: Option<Alpha3>,
}

impl Validate for NewProducts {
    fn validate(&self) -> Result<(), ValidationErrors> {
        // TODO: Also validate measurements when the price is specified (will break frontend at the moment)
        if self.price.is_none() && self.measurements.is_none() {
            Err(validation_errors!({ "measurements": ["measurements" => "Measurements must be specified"] }))?;
        }

        if let Some(ref measurements) = self.measurements {
            measurements.validate()?;
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewProductValidation {
    pub product: NewProducts,
    pub company: Company,
    pub package: Packages,
}

impl Validate for NewProductValidation {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let NewProductValidation {
            ref product,
            ref package,
            ref company,
        } = self;
        product.validate()?;

        let NewProducts {
            ref measurements,
            delivery_from,
            ref deliveries_to,
            ..
        } = product;

        if let Some(delivery_from) = delivery_from {
            if get_country_from_forest(company.deliveries_from.iter(), delivery_from).is_none() {
                let msg = format!("Delivery from {} is not available for company {}", delivery_from, company.name);
                Err(validation_errors!({
                    "delivery_from": ["delivery_from" => msg]
                }))?
            }
        }

        let unavailable_destinations = deliveries_to
            .iter()
            .filter_map(|prod_alpha3| {
                if get_country_from_forest(package.deliveries_to.iter(), prod_alpha3).is_none() {
                    Some(prod_alpha3.clone().0)
                } else {
                    None
                }
            }).collect::<Vec<_>>();

        println!("{:?}", unavailable_destinations);

        if !unavailable_destinations.is_empty() {
            let unavailable_destinations = unavailable_destinations.as_slice().join(", ");
            let msg = format!(
                "Delivery to {} is not available for company {}",
                unavailable_destinations, company.name
            );
            Err(validation_errors!({
                "deliveries_to": ["deliveries_to" => msg]
            }))?
        }

        if let Some(measurements) = measurements {
            package.within_limits(*measurements).map_err(|err| {
                let mut validation_error = ValidationError::new("measurements");
                validation_error.add_param("measurements".into(), &err);

                let mut validation_errors = ValidationErrors::new();
                validation_errors.add("shipment", validation_error);

                validation_errors
            })?;
        }

        Ok(())
    }
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
