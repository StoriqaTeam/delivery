use failure::Error as FailureError;

use stq_types::{Alpha3, CompanyPackageId, ShippingRatesId};

use models::ShipmentMeasurements;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct ShippingRate {
    pub weight_g: u32,
    pub price: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ShippingRates {
    pub id: ShippingRatesId,
    pub company_package_id: CompanyPackageId,
    pub from_alpha3: Alpha3,
    pub to_alpha3: Alpha3,
    pub rates: Vec<ShippingRate>,
}

impl ShippingRates {
    pub fn calculate_delivery_price(&self, measurements: ShipmentMeasurements, dimensional_factor: Option<u32>) -> Option<f64> {
        let billable_weight_g = measurements.calculate_billable_weight(dimensional_factor);
        super::calculate_delivery_price(billable_weight_g, self.rates.clone())
    }
}

pub fn calculate_delivery_price(billable_weight_g: u32, mut rates: Vec<ShippingRate>) -> Option<f64> {
    rates.sort_unstable_by_key(|rate| rate.weight_g);

    rates
        .into_iter()
        .find(|rate| rate.weight_g >= billable_weight_g)
        .map(|rate| rate.price)
}

#[derive(Clone, Serialize, Associations, Queryable, Debug)]
#[table_name = "shipping_rates"]
pub struct ShippingRatesRaw {
    pub id: ShippingRatesId,
    pub company_package_id: CompanyPackageId,
    pub from_alpha3: Alpha3,
    pub to_alpha3: Alpha3,
    pub rates: serde_json::Value,
}

impl ShippingRatesRaw {
    pub fn to_model(self) -> Result<ShippingRates, FailureError> {
        let ShippingRatesRaw {
            id,
            company_package_id,
            from_alpha3,
            to_alpha3,
            rates,
        } = self;

        serde_json::from_value::<Vec<ShippingRate>>(rates)
            .map_err(|e| {
                FailureError::from(e)
                    .context(format!("Could not parse JSON with rates for ShippingRates with id = {}", id))
                    .into()
            }).map(|rates| ShippingRates {
                id,
                company_package_id,
                from_alpha3,
                to_alpha3,
                rates,
            })
    }
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
        let shipping_rates = ShippingRates {
            id: ShippingRatesId(1),
            company_package_id: CompanyPackageId(1),
            from_alpha3: Alpha3("RUS".to_string()),
            to_alpha3: Alpha3("USA".to_string()),
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
            shipping_rates.clone().calculate_delivery_price(
                ShipmentMeasurements {
                    volume_cubic_cm: 1000,
                    weight_g: 100
                },
                Some(5)
            ),
        );

        assert_eq!(
            Some(1200.0),
            shipping_rates.clone().calculate_delivery_price(
                ShipmentMeasurements {
                    volume_cubic_cm: 1000,
                    weight_g: 600
                },
                Some(5)
            ),
        );

        assert_eq!(
            Some(1200.0),
            shipping_rates.clone().calculate_delivery_price(
                ShipmentMeasurements {
                    volume_cubic_cm: 3000,
                    weight_g: 100
                },
                Some(5)
            ),
        );

        assert_eq!(
            None,
            shipping_rates.clone().calculate_delivery_price(
                ShipmentMeasurements {
                    volume_cubic_cm: 3000,
                    weight_g: 1001
                },
                Some(5)
            ),
        );

        assert_eq!(
            Some(600.0),
            shipping_rates.clone().calculate_delivery_price(
                ShipmentMeasurements {
                    volume_cubic_cm: 9999,
                    weight_g: 1
                },
                None
            ),
        );
    }
}
