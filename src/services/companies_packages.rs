//! CompaniesPackages Service, presents CRUD operations

use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Error as FailureError;
use r2d2::ManageConnection;
use stq_static_resources::Currency;
use stq_types::{Alpha3, CompanyId, CompanyPackageId, PackageId};
use validator::Validate;

use errors::Error;
use models::{
    get_countries_from_forest_by, AvailablePackages, Company, CompanyPackage, Country, NewCompanyPackage, NewShippingRates,
    NewShippingRatesBatch, PackageValidation, Packages, RatesCsvData, ShipmentMeasurements, ShippingRateSource, ShippingRates,
    ShippingValidation, ZonesCsvData,
};
use repos::ReposFactory;
use services::types::{Service, ServiceFuture};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetDeliveryPrice {
    pub company_package_id: CompanyPackageId,
    pub delivery_from: Alpha3,
    pub delivery_to: Alpha3,
    pub volume: u32,
    pub weight: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DeliveryPrice {
    pub currency: Currency,
    pub value: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ReplaceShippingRatesPayload {
    pub rates_csv_base64: String,
    pub zones_csv_base64: String,
}

pub trait CompaniesPackagesService {
    /// Create a new companies_packages
    fn create_company_package(&self, payload: NewCompanyPackage) -> ServiceFuture<CompanyPackage>;

    /// Returns available packages supported by the country
    fn get_available_packages(&self, country: Alpha3, size: u32, weight: u32) -> ServiceFuture<Vec<AvailablePackages>>;

    /// Returns company package by id
    fn get_company_package(&self, id: CompanyPackageId) -> ServiceFuture<Option<CompanyPackage>>;

    /// Returns companies by package id
    fn get_companies(&self, id: PackageId) -> ServiceFuture<Vec<Company>>;

    /// Returns packages by company id
    fn get_packages(&self, id: CompanyId) -> ServiceFuture<Vec<Packages>>;

    /// Delete a companies_packages
    fn delete_company_package(&self, company_id: CompanyId, package_id: PackageId) -> ServiceFuture<CompanyPackage>;

    /// Get delivery price
    fn get_delivery_price(&self, payload: GetDeliveryPrice) -> ServiceFuture<Option<DeliveryPrice>>;

    /// Get shipping rates for the particular "from" country in the company package
    fn get_shipping_rates(&self, company_package_id: CompanyPackageId, delivery_from: Alpha3) -> ServiceFuture<Vec<ShippingRates>>;

    /// Replace shipping rates for the particular "from" country in the company package
    fn replace_shipping_rates(
        &self,
        company_package_id: CompanyPackageId,
        payload: ReplaceShippingRatesPayload,
    ) -> ServiceFuture<Vec<ShippingRates>>;
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > CompaniesPackagesService for Service<T, M, F>
{
    /// Create a new companies_packages
    fn create_company_package(&self, payload: NewCompanyPackage) -> ServiceFuture<CompanyPackage> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let companies_packages_repo = repo_factory.create_companies_packages_repo(&*conn, user_id);
            conn.transaction::<CompanyPackage, FailureError, _>(move || {
                companies_packages_repo
                    .create(payload)
                    .map_err(|e| e.context("Service CompaniesPackages, create endpoint error occured.").into())
            })
        })
    }

    /// Returns company package by id
    fn get_company_package(&self, id: CompanyPackageId) -> ServiceFuture<Option<CompanyPackage>> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let companies_packages_repo = repo_factory.create_companies_packages_repo(&*conn, user_id);
            companies_packages_repo
                .get(id)
                .map_err(|e| e.context("Service CompaniesPackages, get endpoint error occured.").into())
        })
    }

    /// Returns companies by package id
    fn get_companies(&self, id: PackageId) -> ServiceFuture<Vec<Company>> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let companies_packages_repo = repo_factory.create_companies_packages_repo(&*conn, user_id);
            companies_packages_repo
                .get_companies(id)
                .map_err(|e| e.context("Service CompaniesPackages, get_companies endpoint error occured.").into())
        })
    }

    /// Returns packages by company id
    fn get_packages(&self, id: CompanyId) -> ServiceFuture<Vec<Packages>> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let companies_packages_repo = repo_factory.create_companies_packages_repo(&*conn, user_id);
            companies_packages_repo
                .get_packages(id)
                .map_err(|e| e.context("Service CompaniesPackages, get_packages endpoint error occured.").into())
        })
    }

    /// Returns list of companies_packages supported by the country
    fn get_available_packages(&self, deliveries_from: Alpha3, size: u32, weight: u32) -> ServiceFuture<Vec<AvailablePackages>> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let companies_repo = repo_factory.create_companies_repo(&*conn, user_id);
            let companies_packages_repo = repo_factory.create_companies_packages_repo(&*conn, user_id);
            let shipping_rates_repo = repo_factory.create_shipping_rates_repo(&*conn, user_id);

            companies_repo
                .find_deliveries_from(deliveries_from.clone())
                .and_then(|companies| {
                    let companies_ids = companies.into_iter().map(|company| company.id).collect();
                    companies_packages_repo
                        .get_available_packages(companies_ids, size, weight, deliveries_from.clone())?
                        .into_iter()
                        .map(|pkg| {
                            let deliveries_to =
                                get_countries_from_forest_by(pkg.deliveries_to.iter(), |country| country.level == Country::COUNTRY_LEVEL)
                                    .into_iter()
                                    .map(|country| country.alpha3)
                                    .collect::<Vec<_>>();

                            match pkg.shipping_rate_source {
                                ShippingRateSource::NotAvailable => Ok((pkg, None)),
                                ShippingRateSource::Static { dimensional_factor } => shipping_rates_repo
                                    .get_multiple_rates(pkg.id, deliveries_from.clone(), deliveries_to)
                                    .map(move |rates| (pkg, Some((dimensional_factor, rates)))),
                            }
                        })
                        .collect::<Result<Vec<_>, _>>()
                        .map(|package_rates| {
                            package_rates
                                .into_iter()
                                .filter_map(|(pkg, rates)| determine_package_availability(rates, size, weight, pkg))
                                .collect::<Vec<_>>()
                        })
                })
                .map_err(|e| {
                    e.context("Service CompaniesPackages, find_deliveries_from endpoint error occured.")
                        .into()
                })
        })
    }

    /// Delete a companies_packages
    fn delete_company_package(&self, company_id: CompanyId, package_id: PackageId) -> ServiceFuture<CompanyPackage> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let companies_packages_repo = repo_factory.create_companies_packages_repo(&*conn, user_id);
            companies_packages_repo
                .delete(company_id, package_id)
                .map_err(|e| e.context("Service CompaniesPackages, delete endpoint error occured.").into())
        })
    }

    /// Get delivery price
    fn get_delivery_price(&self, payload: GetDeliveryPrice) -> ServiceFuture<Option<DeliveryPrice>> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        let GetDeliveryPrice {
            company_package_id,
            volume,
            weight,
            delivery_from,
            delivery_to,
        } = payload;

        let measurements = ShipmentMeasurements {
            volume_cubic_cm: volume,
            weight_g: weight,
        };

        self.spawn_on_pool(move |conn| {
            let companies_repo = repo_factory.create_companies_repo(&*conn, user_id);
            let packages_repo = repo_factory.create_packages_repo(&*conn, user_id);
            let companies_packages_repo = repo_factory.create_companies_packages_repo(&*conn, user_id);
            let shipping_rates_repo = repo_factory.create_shipping_rates_repo(&*conn, user_id);

            let run = move || {
                let company_package = companies_packages_repo
                    .get(company_package_id)?
                    .ok_or(Error::Validate(validation_errors!({
                        "company_package": ["company_package" => "Company package not found"]
                    })))?;

                let delivery_price = match company_package.shipping_rate_source.clone() {
                    ShippingRateSource::NotAvailable => None,
                    ShippingRateSource::Static { dimensional_factor } => {
                        let company = companies_repo
                            .find(company_package.company_id)?
                            .ok_or(format_err!("Company with id {} not found", company_package.company_id))?;

                        let package = packages_repo
                            .find(company_package.package_id)?
                            .ok_or(format_err!("Package with id {} not found", company_package.package_id))?;

                        PackageValidation {
                            measurements: measurements.clone(),
                            package: package.clone(),
                        }
                        .validate()
                        .map_err(Error::Validate)?;

                        let currency = company.currency;

                        let shipping_available = ShippingValidation {
                            delivery_from: Some(delivery_from.clone()),
                            deliveries_to: vec![delivery_to.clone()],
                            company,
                            package,
                        }
                        .validate()
                        .is_ok();

                        if !shipping_available {
                            None
                        } else {
                            shipping_rates_repo
                                .get_rates(company_package_id, delivery_from, delivery_to)?
                                .and_then(|rates| {
                                    rates
                                        .calculate_delivery_price(measurements, dimensional_factor)
                                        .map(|price| DeliveryPrice { currency, value: price })
                                })
                        }
                    }
                };

                Ok(delivery_price)
            };

            run().map_err(|e: FailureError| {
                e.context("Service CompaniesPackages, get_delivery_price endpoint error occurred.")
                    .into()
            })
        })
    }

    /// Get shipping rates for the particular "from" country in the company package
    fn get_shipping_rates(&self, company_package_id: CompanyPackageId, delivery_from: Alpha3) -> ServiceFuture<Vec<ShippingRates>> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let shipping_rates_repo = repo_factory.create_shipping_rates_repo(&*conn, user_id);
            shipping_rates_repo
                .get_all_rates_from(company_package_id, delivery_from)
                .map_err(|e| {
                    e.context("Service CompaniesPackages, get_shipping_rates endpoint error occured.")
                        .into()
                })
        })
    }

    /// Replace shipping rates for the particular "from" country in the company package
    fn replace_shipping_rates(
        &self,
        company_package_id: CompanyPackageId,
        payload: ReplaceShippingRatesPayload,
    ) -> ServiceFuture<Vec<ShippingRates>> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let ReplaceShippingRatesPayload {
                rates_csv_base64,
                zones_csv_base64,
            } = payload;

            let rates = base64::decode(&rates_csv_base64)
                .map_err(|_| {
                    let errors = validation_errors!({ "payload": ["rates_csv_base64" => "Failed to decode base64 rates CSV"] });
                    Error::Validate(errors).into()
                })
                .and_then(|csv| {
                    RatesCsvData::parse_csv(csv.as_slice()).map_err(|e| {
                        let errors = validation_errors!({ "payload": ["rates_csv_base64" => e.to_string()] });
                        FailureError::from(Error::Validate(errors))
                    })
                })?;

            let zones = base64::decode(&zones_csv_base64)
                .map_err(|_| {
                    let errors = validation_errors!({ "payload": ["zones_csv_base64" => "Failed to decode base64 zones CSV"] });
                    Error::Validate(errors).into()
                })
                .and_then(|csv| {
                    ZonesCsvData::parse_csv(csv.as_slice()).map_err(|e| {
                        let errors = validation_errors!({ "payload": ["zones_csv_base64" => e.to_string()] });
                        FailureError::from(Error::Validate(errors))
                    })
                })?;

            let NewShippingRatesBatch {
                company_package_id,
                delivery_from,
                delivery_to_rates,
            } = NewShippingRatesBatch::try_from_csv_data(company_package_id, zones, rates).map_err(|e| {
                let errors = validation_errors!({ "payload": ["payload" => e.to_string()] });
                FailureError::from(Error::Validate(errors))
            })?;

            let new_shipping_rates = delivery_to_rates
                .into_iter()
                .map(|(to_alpha3, rates)| NewShippingRates {
                    company_package_id: company_package_id.clone(),
                    from_alpha3: delivery_from.clone(),
                    to_alpha3,
                    rates,
                })
                .collect::<Vec<_>>();

            let companies_packages_repo = repo_factory.create_companies_packages_repo(&*conn, user_id);
            let shipping_rates_repo = repo_factory.create_shipping_rates_repo(&*conn, user_id);

            companies_packages_repo
                .get(company_package_id)
                .map_err(|e| FailureError::from(e.context("Service CompaniesPackages, replace_shipping_rates endpoint error occured.")))?
                .ok_or(format_err!("Company package with id = {} not found", company_package_id))?;

            conn.transaction::<Vec<ShippingRates>, FailureError, _>(move || {
                shipping_rates_repo.delete_all_rates_from(company_package_id, delivery_from)?;
                shipping_rates_repo.insert_many(new_shipping_rates)
            })
            .map_err(|e| {
                e.context("Service CompaniesPackages, replace_shipping_rates endpoint error occured.")
                    .into()
            })
        })
    }
}

fn determine_package_availability(
    rates: Option<(Option<u32>, Vec<ShippingRates>)>,
    volume: u32,
    weight: u32,
    mut pkg: AvailablePackages,
) -> Option<AvailablePackages> {
    match rates {
        // If the company-package does not have static shipping rates,
        // it is available for fixed price delivery
        None => Some(pkg),
        // If the company-package has static shipping rates,
        // they are also used to determine whether the delivery is avaliable
        Some((dimensional_factor, rates)) => {
            let serviced_dest_countries = rates
                .into_iter()
                .filter_map(|rates| {
                    let measurements = ShipmentMeasurements {
                        volume_cubic_cm: volume,
                        weight_g: weight,
                    };
                    rates
                        .calculate_delivery_price(measurements, dimensional_factor)
                        .map(move |_| rates.to_alpha3)
                })
                .collect::<Vec<_>>();

            let available_dest_countries = get_countries_from_forest_by(pkg.deliveries_to.iter(), |country| {
                serviced_dest_countries
                    .iter()
                    .any(|serviced_country_alpha3| country.alpha3 == *serviced_country_alpha3)
            });

            if available_dest_countries.is_empty() {
                None
            } else {
                pkg.deliveries_to = available_dest_countries;
                Some(pkg)
            }
        }
    }
}
