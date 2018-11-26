use std::cmp::max;

use failure::Error as FailureError;
use validator::{Validate, ValidationErrors};

use models::{Country, Pickups, ShippingVariant};
use stq_static_resources::Currency;
use stq_types::{BaseProductId, CompanyId, CompanyPackageId, PackageId, ProductPrice, ShippingId, StoreId};

use schema::companies_packages;

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct ShipmentMeasurements {
    pub volume_cubic_cm: u32,
    pub weight_g: u32,
}

impl ShipmentMeasurements {
    pub fn calculate_billable_weight(&self, dimensional_factor: Option<u32>) -> u32 {
        let ShipmentMeasurements { volume_cubic_cm, weight_g } = self;

        match dimensional_factor.filter(|df| *df > 0) {
            None => *weight_g,
            Some(dimensional_factor) => {
                let dim_weight = f64::ceil(*volume_cubic_cm as f64 / dimensional_factor as f64) as u32;
                max(*weight_g, dim_weight)
            }
        }
    }
}

impl Validate for ShipmentMeasurements {
    fn validate(&self) -> Result<(), ValidationErrors> {
        const MAX_REASONABLE_VOLUME_CUBIC_CM: u32 = 2_000_000;
        const MAX_REASONABLE_WEIGHT_G: u32 = 100_000;

        if !(self.volume_cubic_cm > 0 && self.volume_cubic_cm <= MAX_REASONABLE_VOLUME_CUBIC_CM) {
            Err(validation_errors!({ "volume_cubic_cm": ["volume_cubic_cm" => "Volume must be in range 0 < x <= 2 000 000 cm^3"] }))?;
        }

        if !(self.weight_g > 0 && self.weight_g <= MAX_REASONABLE_WEIGHT_G) {
            Err(validation_errors!({ "weight_g": ["weight_g" => "Weight must be in range 0 < x <= 100 000 g"] }))?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ShippingRates {
    dimensional_factor: Option<u32>,
    rates: Vec<ShippingRate>,
}

impl ShippingRates {
    pub fn calculate_delivery_price(self, measurements: ShipmentMeasurements) -> Option<f64> {
        let billable_weight_g = measurements.calculate_billable_weight(self.dimensional_factor);
        super::calculate_delivery_price(billable_weight_g, self.rates)
    }
}

pub fn calculate_delivery_price(billable_weight_g: u32, mut rates: Vec<ShippingRate>) -> Option<f64> {
    rates.sort_unstable_by_key(|rate| rate.weight_g);

    rates
        .into_iter()
        .find(|rate| rate.weight_g >= billable_weight_g)
        .map(|rate| rate.price)
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct ShippingRate {
    weight_g: u32,
    price: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ShippingRateSource {
    NotAvailable,
    Static { dimensional_factor: Option<u32> },
}

impl Default for ShippingRateSource {
    fn default() -> Self {
        ShippingRateSource::NotAvailable
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Debug, DieselTypes)]
pub enum ShippingRateSourceRaw {
    NotAvailable,
    Static,
    OnDemand,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CompanyPackage {
    pub id: CompanyPackageId,
    pub company_id: CompanyId,
    pub package_id: PackageId,
    pub shipping_rate_source: ShippingRateSource,
}

#[derive(Serialize, Deserialize, Associations, Queryable, Debug)]
#[table_name = "companies_packages"]
pub struct CompaniesPackagesRaw {
    pub id: CompanyPackageId,
    pub company_id: CompanyId,
    pub package_id: PackageId,
    pub shipping_rate_source: ShippingRateSourceRaw,
    pub dimensional_factor: Option<i32>,
}

impl CompaniesPackagesRaw {
    pub fn to_model(self) -> Result<CompanyPackage, FailureError> {
        let CompaniesPackagesRaw {
            id,
            company_id,
            package_id,
            shipping_rate_source,
            dimensional_factor,
        } = self;

        match shipping_rate_source {
            ShippingRateSourceRaw::NotAvailable => Ok(CompanyPackage {
                id,
                company_id,
                package_id,
                shipping_rate_source: ShippingRateSource::NotAvailable,
            }),
            ShippingRateSourceRaw::Static => match dimensional_factor {
                None => Ok(CompanyPackage {
                    id,
                    company_id,
                    package_id,
                    shipping_rate_source: ShippingRateSource::Static { dimensional_factor: None },
                }),
                Some(dimensional_factor) => if dimensional_factor < 0 {
                    Err(format_err!("Negative dimensional factor value for CompanyPackage with id = {}", id))
                } else {
                    Ok(CompanyPackage {
                        id,
                        company_id,
                        package_id,
                        shipping_rate_source: ShippingRateSource::Static {
                            dimensional_factor: Some(dimensional_factor as u32),
                        },
                    })
                },
            },
            ShippingRateSourceRaw::OnDemand => Err(format_err!(
                "CompanyPackages with on-demand sources of shipping rates \
                 are not yet supported (CompanyPackage id = {})",
                id
            )),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NewCompanyPackage {
    pub company_id: CompanyId,
    pub package_id: PackageId,
    pub shipping_rate_source: Option<ShippingRateSource>,
}

#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "companies_packages"]
pub struct NewCompaniesPackagesRaw {
    pub company_id: CompanyId,
    pub package_id: PackageId,
    pub shipping_rate_source: ShippingRateSourceRaw,
    pub dimensional_factor: Option<i32>,
}

impl From<NewCompanyPackage> for NewCompaniesPackagesRaw {
    fn from(new_company_package: NewCompanyPackage) -> Self {
        let NewCompanyPackage {
            company_id,
            package_id,
            shipping_rate_source,
        } = new_company_package;

        match shipping_rate_source.unwrap_or_default() {
            ShippingRateSource::NotAvailable => NewCompaniesPackagesRaw {
                company_id,
                package_id,
                shipping_rate_source: ShippingRateSourceRaw::NotAvailable,
                dimensional_factor: None,
            },
            ShippingRateSource::Static { dimensional_factor } => NewCompaniesPackagesRaw {
                company_id,
                package_id,
                shipping_rate_source: ShippingRateSourceRaw::Static,
                dimensional_factor: dimensional_factor.map(|df| df as i32),
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AvailablePackages {
    pub id: CompanyPackageId,
    pub name: String,
    pub logo: String,
    pub deliveries_to: Vec<Country>,
    pub currency: Currency,
    pub local_available: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AvailablePackageForUser {
    pub id: CompanyPackageId,
    pub shipping_id: ShippingId,
    pub name: String,
    pub logo: String,
    pub price: Option<ProductPrice>,
    pub shipping_variant: ShippingVariant,
    pub deliveries_to: Vec<Country>,
    pub base_product_id: BaseProductId,
    pub store_id: StoreId,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AvailableShippingForUser {
    pub packages: Vec<AvailablePackageForUser>,
    pub pickups: Option<Pickups>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_billable_weight_dimensional_weight_is_chosen() {
        let dimensional_factor = Some(5);
        let measurements = ShipmentMeasurements {
            volume_cubic_cm: 1000,
            weight_g: 12,
        };

        let expected_billable_weight = 1000 / 5;

        assert_eq!(expected_billable_weight, measurements.calculate_billable_weight(dimensional_factor));
    }

    #[test]
    fn calculate_billable_weight_physical_weight_is_chosen() {
        let dimensional_factor = Some(5);
        let measurements = ShipmentMeasurements {
            volume_cubic_cm: 10,
            weight_g: 12,
        };

        let expected_billable_weight = 12;

        assert_eq!(expected_billable_weight, measurements.calculate_billable_weight(dimensional_factor));
    }

    #[test]
    fn calculate_billable_weight_dimensional_weight_rounds_up() {
        let dimensional_factor = Some(6);
        let measurements = ShipmentMeasurements {
            volume_cubic_cm: 1000,
            weight_g: 12,
        };

        let expected_billable_weight = 1000 / 6 + 1;

        assert_eq!(expected_billable_weight, measurements.calculate_billable_weight(dimensional_factor));
    }

    #[test]
    fn calculate_billable_weight_no_dimensional_factor_physical_weight_is_chosen() {
        let dimensional_factor = None;
        let measurements = ShipmentMeasurements {
            volume_cubic_cm: 1000,
            weight_g: 12,
        };

        let expected_billable_weight = 12;
        assert_eq!(expected_billable_weight, measurements.calculate_billable_weight(dimensional_factor));
    }

    #[test]
    fn calculate_billable_weight_zero_dimensional_factor_physical_weight_is_chosen() {
        let dimensional_factor = Some(0);
        let measurements = ShipmentMeasurements {
            volume_cubic_cm: 1000,
            weight_g: 12,
        };

        let expected_billable_weight = 12;
        assert_eq!(expected_billable_weight, measurements.calculate_billable_weight(dimensional_factor));
    }

    #[test]
    fn calculate_delivery_price_empty_rates() {
        assert_eq!(None, calculate_delivery_price(0, vec![]));
        assert_eq!(None, calculate_delivery_price(1, vec![]));
    }

    #[test]
    fn calculate_delivery_price_single_rate() {
        let rates = vec![ShippingRate {
            weight_g: 1000,
            price: 1200.0,
        }];
        assert_eq!(Some(1200.0), calculate_delivery_price(0, rates.clone()));
        assert_eq!(Some(1200.0), calculate_delivery_price(1, rates.clone()));
        assert_eq!(Some(1200.0), calculate_delivery_price(1000, rates.clone()));
        assert_eq!(None, calculate_delivery_price(1001, rates.clone()));
    }

    #[test]
    fn calculate_delivery_price_common_cases() {
        let rates = vec![
            ShippingRate {
                weight_g: 1000,
                price: 1200.0,
            },
            ShippingRate {
                weight_g: 500,
                price: 900.0,
            },
            ShippingRate {
                weight_g: 1500,
                price: 1400.0,
            },
        ];

        assert_eq!(Some(900.0), calculate_delivery_price(0, rates.clone()));
        assert_eq!(Some(900.0), calculate_delivery_price(1, rates.clone()));
        assert_eq!(Some(900.0), calculate_delivery_price(499, rates.clone()));
        assert_eq!(Some(900.0), calculate_delivery_price(500, rates.clone()));
        assert_eq!(Some(1200.0), calculate_delivery_price(501, rates.clone()));
        assert_eq!(Some(1200.0), calculate_delivery_price(999, rates.clone()));
        assert_eq!(Some(1200.0), calculate_delivery_price(1000, rates.clone()));
        assert_eq!(Some(1400.0), calculate_delivery_price(1001, rates.clone()));
        assert_eq!(Some(1400.0), calculate_delivery_price(1499, rates.clone()));
        assert_eq!(Some(1400.0), calculate_delivery_price(1500, rates.clone()));
        assert_eq!(None, calculate_delivery_price(1501, rates));
    }

    #[test]
    fn shipping_rates_calculate_delivery_rates() {
        let mut shipping_rates = ShippingRates {
            dimensional_factor: Some(5),
            rates: vec![
                ShippingRate {
                    weight_g: 500,
                    price: 600.0,
                },
                ShippingRate {
                    weight_g: 1000,
                    price: 1200.0,
                },
            ],
        };

        assert_eq!(
            Some(600.0),
            shipping_rates.clone().calculate_delivery_price(ShipmentMeasurements {
                volume_cubic_cm: 1000,
                weight_g: 100
            }),
        );

        assert_eq!(
            Some(1200.0),
            shipping_rates.clone().calculate_delivery_price(ShipmentMeasurements {
                volume_cubic_cm: 1000,
                weight_g: 600
            }),
        );

        assert_eq!(
            Some(1200.0),
            shipping_rates.clone().calculate_delivery_price(ShipmentMeasurements {
                volume_cubic_cm: 3000,
                weight_g: 100
            }),
        );

        assert_eq!(
            None,
            shipping_rates.clone().calculate_delivery_price(ShipmentMeasurements {
                volume_cubic_cm: 3000,
                weight_g: 1001
            }),
        );

        shipping_rates.dimensional_factor = None;

        assert_eq!(
            Some(600.0),
            shipping_rates.clone().calculate_delivery_price(ShipmentMeasurements {
                volume_cubic_cm: 9999,
                weight_g: 1
            }),
        );
    }
}
