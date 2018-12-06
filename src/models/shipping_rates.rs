use failure::{err_msg, Error as FailureError, Fail};
use std::collections::HashMap;
use std::str::FromStr;

use stq_types::{Alpha3, CompanyPackageId, ShippingRatesId};

use models::ShipmentMeasurements;
use schema::shipping_rates;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
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

pub struct NewShippingRates {
    pub company_package_id: CompanyPackageId,
    pub from_alpha3: Alpha3,
    pub to_alpha3: Alpha3,
    pub rates: Vec<ShippingRate>,
}

#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "shipping_rates"]
pub struct NewShippingRatesRaw {
    pub company_package_id: CompanyPackageId,
    pub from_alpha3: Alpha3,
    pub to_alpha3: Alpha3,
    pub rates: serde_json::Value,
}

impl NewShippingRatesRaw {
    pub fn from_batch(batch: NewShippingRatesBatch) -> Result<Vec<Self>, FailureError> {
        let NewShippingRatesBatch {
            company_package_id,
            delivery_from,
            delivery_to_rates,
        } = batch;
        delivery_to_rates
            .into_iter()
            .map(|(to_alpha3, rates)| {
                serde_json::to_value(rates)
                    .map_err(FailureError::from)
                    .map(|rates| NewShippingRatesRaw {
                        company_package_id,
                        from_alpha3: delivery_from.clone(),
                        to_alpha3: to_alpha3.clone(),
                        rates,
                    })
            }).collect()
    }
}

impl NewShippingRatesRaw {
    pub fn from_model(new_shipping_rates: NewShippingRates) -> Result<Self, FailureError> {
        let NewShippingRates {
            company_package_id,
            from_alpha3,
            to_alpha3,
            rates,
        } = new_shipping_rates;

        let rates = serde_json::to_value(&rates).map_err(FailureError::from)?;

        Ok(NewShippingRatesRaw {
            company_package_id,
            from_alpha3,
            to_alpha3,
            rates,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ZonesCsvEntry {
    pub from: Alpha3,
    pub to: Alpha3,
    pub zone: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ZonesCsvData(pub Vec<ZonesCsvEntry>);

impl ZonesCsvData {
    /// https://storiqa.atlassian.net/wiki/spaces/PROD/pages/475791364?preview=/475791364/516620310/Russian%20export%20-%20CountryToZone.csv
    pub fn parse_csv(csv: &[u8]) -> Result<ZonesCsvData, FailureError> {
        let mut reader = csv::Reader::from_reader(csv);

        let data = reader
            .records()
            .enumerate()
            .try_fold(Vec::<ZonesCsvEntry>::new(), |mut entries, (row_num, record)| {
                let row_num = row_num + 2; // Count from 1, skip header row
                let record = record.map_err(|e| FailureError::from(e.context(format!("Invalid CSV record (row {})", row_num))))?;

                match record.iter().map(String::from).collect::<Vec<_>>().as_mut_slice() {
                    [ref mut from, ref mut to, ref zone] => {
                        from.make_ascii_uppercase();
                        if from.len() != 3 || from.chars().any(|c| !c.is_alphabetic()) {
                            Err(format_err!("Invalid ISO alpha 3 country code (row {}, column 1)", row_num))?;
                        }
                        let from = Alpha3(from.to_string());

                        to.make_ascii_uppercase();
                        if to.len() != 3 || to.chars().any(|c| !c.is_alphabetic()) {
                            Err(format_err!("Invalid ISO alpha 3 country code (row {}, column 2)", row_num))?;
                        }
                        let to = Alpha3(to.to_string());

                        let zone = u32::from_str(&zone).map_err(|e| {
                            FailureError::from(e.context(format!("Invalid zone number format (row {}, column 3)", row_num)))
                        })?;

                        if let Some(e) = entries.iter().find(|e| e.from == from && e.to == to && e.zone != zone) {
                            Err(format_err!(
                                "Found conflicting entries \"{},{},{}\" and \"{},{},{}\"",
                                &e.from,
                                &e.to,
                                e.zone,
                                from,
                                to,
                                zone,
                            ))?;
                        }

                        if !entries.iter().any(|e| e.from == from && e.to == to && e.zone == zone) {
                            entries.push(ZonesCsvEntry { from, to, zone });
                        }

                        Ok(entries)
                    }
                    _ => Err(format_err!("Invalid row {}", row_num)),
                }
            })?;

        if data.is_empty() {
            Err(err_msg("CSV is empty"))
        } else {
            Ok(ZonesCsvData(data))
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RatesCsvData(pub HashMap<u32, Vec<ShippingRate>>);

impl RatesCsvData {
    /// https://storiqa.atlassian.net/wiki/spaces/PROD/pages/475791364?preview=/475791364/516587537/Russian%20export%20-%20UPS%20Express%20Saver.csv
    pub fn parse_csv(csv: &[u8]) -> Result<RatesCsvData, FailureError> {
        let mut reader = csv::Reader::from_reader(csv);
        let mut records = reader.records();

        let zones = records
            .next()
            .ok_or(err_msg("Row with zone numbers not found"))?
            .map_err(|e| FailureError::from(e.context("Row 2 has invalid format")))?
            .iter()
            .skip(1) // zone numbers start from 2nd column
            .map(u32::from_str)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| FailureError::from(e.context("Zone numbers have invalid format")))?;

        if zones.is_empty() {
            Err(err_msg("Zone numbers row is empty"))?;
        }

        let mut shipping_rates_for_zones = HashMap::<u32, Vec<ShippingRate>>::new();
        for (row_num, record) in records.enumerate() {
            let row_num = row_num + 3; // Count from 1, skip header row, skip zones row
            let record = record.map_err(|e| FailureError::from(e.context(format!("Row {} has invalid format", row_num))))?;

            if record.len() != 1 + zones.len() {
                Err(format_err!(
                    "Row {} has {} columns, expected {}",
                    row_num,
                    record.len(),
                    1 + zones.len()
                ))?;
            }

            let mut record_iter = record.iter();

            let weight_kg = f64::from_str(record_iter.next().ok_or(err_msg("Unexpected error"))?)
                .map_err(|e| FailureError::from(e.context(format!("Invalid weight format (row {})", row_num))))?;

            for (i, (zone_price, zone_num)) in record_iter.zip(zones.clone()).enumerate() {
                let col_num = i + 2; // Count from 1, skip weight column
                let zone_price = f64::from_str(zone_price)
                    .map_err(|e| FailureError::from(e.context(format!("Invalid price format (row {}, column {})", row_num, col_num))))?;

                let shipping_rate = ShippingRate {
                    weight_g: f64::round(weight_kg * 1000.0) as u32,
                    price: zone_price,
                };

                let rates = shipping_rates_for_zones.entry(zone_num).or_insert(Vec::new());
                (*rates).push(shipping_rate);
            }
        }

        Ok(RatesCsvData(shipping_rates_for_zones))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct NewShippingRatesBatch {
    pub company_package_id: CompanyPackageId,
    pub delivery_from: Alpha3,
    pub delivery_to_rates: Vec<(Alpha3, Vec<ShippingRate>)>,
}

impl NewShippingRatesBatch {
    pub fn try_from_csv_data(
        company_package_id: CompanyPackageId,
        zones: ZonesCsvData,
        rates: RatesCsvData,
    ) -> Result<NewShippingRatesBatch, FailureError> {
        if zones.0.is_empty() {
            Err(err_msg("Zone table is empty"))?;
        };

        if rates.0.is_empty() {
            Err(err_msg("Rate table is empty"))?;
        };

        let from = zones.0[0].from.clone();
        if zones.0.iter().any(|z| z.from != from) {
            Err(err_msg("Zone table must have the same \"from\" country in every row"))?;
        }

        let delivery_to_rates = zones
            .0
            .into_iter()
            .map(|ZonesCsvEntry { to, zone, .. }| {
                rates
                    .0
                    .get(&zone)
                    .cloned()
                    .ok_or(format_err!("Rates for zone {} were not found in the rate table", zone))
                    .map(|rates| (to, rates))
            }).collect::<Result<Vec<_>, _>>()?;

        Ok(NewShippingRatesBatch {
            company_package_id,
            delivery_from: from,
            delivery_to_rates,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::iter::FromIterator;

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

    #[test]
    fn zones_parse_csv_empty() {
        let csv = "From,To,Zone\n".as_bytes();

        ZonesCsvData::parse_csv(csv).unwrap_err();
    }

    #[test]
    fn zones_parse_csv_small() {
        let csv = "From,To,Zone\n\
                   RUS,USA,6\n\
                   ".as_bytes();

        let expected_data = ZonesCsvData(vec![ZonesCsvEntry {
            from: Alpha3("RUS".to_string()),
            to: Alpha3("USA".to_string()),
            zone: 6,
        }]);

        assert_eq!(expected_data, ZonesCsvData::parse_csv(csv).unwrap());
    }

    #[test]
    fn zones_parse_csv_big() {
        let csv = "From,To,Zone\n\
                   RUS,USA,6\n\
                   USA,SGP,7\n\
                   SGP,RUS,6\n\
                   USA,RUS,8\n\
                   ".as_bytes();

        let expected_data = ZonesCsvData(vec![
            ZonesCsvEntry {
                from: Alpha3("RUS".to_string()),
                to: Alpha3("USA".to_string()),
                zone: 6,
            },
            ZonesCsvEntry {
                from: Alpha3("USA".to_string()),
                to: Alpha3("SGP".to_string()),
                zone: 7,
            },
            ZonesCsvEntry {
                from: Alpha3("SGP".to_string()),
                to: Alpha3("RUS".to_string()),
                zone: 6,
            },
            ZonesCsvEntry {
                from: Alpha3("USA".to_string()),
                to: Alpha3("RUS".to_string()),
                zone: 8,
            },
        ]);

        assert_eq!(expected_data, ZonesCsvData::parse_csv(csv).unwrap());
    }

    #[test]
    fn zones_parse_csv_conflicting_entries() {
        let csv = "From,To,Zone\n\
                   RUS,USA,6\n\
                   RUS,USA,7\n\
                   ".as_bytes();

        ZonesCsvData::parse_csv(csv).unwrap_err();
    }

    #[test]
    fn rates_parse_csv_empty() {
        let csv = "Weight,Zone\n".as_bytes();

        RatesCsvData::parse_csv(csv).unwrap_err();
    }

    #[test]
    fn rates_parse_csv_small() {
        let csv = "Weight,Zone\n\
                   ,6\n\
                   0.5,1234.56\n\
                   ".as_bytes();

        let expected_data = RatesCsvData(HashMap::from_iter(vec![(
            6,
            vec![ShippingRate {
                weight_g: 500,
                price: 1234.56,
            }],
        )]));

        assert!(PartialEq::eq(&expected_data, &RatesCsvData::parse_csv(csv).unwrap()))
    }

    #[test]
    fn rates_parse_csv_big() {
        let csv = "Weight,Zone,,\n\
                   ,2,40,800\n\
                   0.5,1,1.2,1.33\n\
                   1,2,2.2,2.33\n\
                   9.99,3,3.2,3.33\n\
                   ".as_bytes();

        let expected_data = RatesCsvData(HashMap::from_iter(vec![
            (
                2,
                vec![
                    ShippingRate { weight_g: 500, price: 1.0 },
                    ShippingRate {
                        weight_g: 1000,
                        price: 2.0,
                    },
                    ShippingRate {
                        weight_g: 9990,
                        price: 3.0,
                    },
                ],
            ),
            (
                40,
                vec![
                    ShippingRate { weight_g: 500, price: 1.2 },
                    ShippingRate {
                        weight_g: 1000,
                        price: 2.2,
                    },
                    ShippingRate {
                        weight_g: 9990,
                        price: 3.2,
                    },
                ],
            ),
            (
                800,
                vec![
                    ShippingRate {
                        weight_g: 500,
                        price: 1.33,
                    },
                    ShippingRate {
                        weight_g: 1000,
                        price: 2.33,
                    },
                    ShippingRate {
                        weight_g: 9990,
                        price: 3.33,
                    },
                ],
            ),
        ]));

        assert!(PartialEq::eq(&expected_data, &RatesCsvData::parse_csv(csv).unwrap()))
    }

    #[test]
    fn rates_parse_csv_invalid() {
        let csv = "Weight,Zone,\n\
                   ,1,2\n\
                   1,1.1,1.2\n\
                   2,2.1,\n\
                   ".as_bytes();

        RatesCsvData::parse_csv(csv).unwrap_err();
    }
}
