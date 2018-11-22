use failure::{Error as FailureError, Fail};

use models::{Country, Pickups, ShippingVariant};
use stq_static_resources::Currency;
use stq_types::{BaseProductId, CompanyId, CompanyPackageId, PackageId, ProductPrice, ShippingId, StoreId};

use schema::companies_packages;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ShippingRates {
    dimensional_factor: f64,
    rates: Vec<ShippingRate>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct ShippingRate {
    weight: f64,
    price: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ShippingRateSource {
    NotAvailable,
    Static(ShippingRates),
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
    pub shipping_rates: Option<serde_json::Value>,
    pub dimensional_factor: Option<f64>,
}

impl CompaniesPackagesRaw {
    pub fn to_model(self) -> Result<CompanyPackage, FailureError> {
        let CompaniesPackagesRaw {
            id,
            company_id,
            package_id,
            shipping_rate_source,
            shipping_rates,
            dimensional_factor,
        } = self;

        match shipping_rate_source {
            ShippingRateSourceRaw::NotAvailable => Ok(CompanyPackage {
                id,
                company_id,
                package_id,
                shipping_rate_source: ShippingRateSource::NotAvailable,
            }),
            ShippingRateSourceRaw::Static => {
                let shipping_rates = shipping_rates
                    .ok_or(format_err!("Shipping rates are missing for CompanyPackage with id = {}", id))
                    .and_then(|rates| {
                        serde_json::from_value::<Vec<ShippingRate>>(rates).map_err(|e| {
                            e.context(format!("Failed to parse shipping rates for CompanyPackage with id = {}", id))
                                .into()
                        })
                    });

                let dimensional_factor =
                    dimensional_factor.ok_or(format_err!("Dimensional factor is missing for CompanyPackage with id = {}", id));

                shipping_rates.and_then(|rates| {
                    dimensional_factor.map(|dimensional_factor| CompanyPackage {
                        id,
                        company_id,
                        package_id,
                        shipping_rate_source: ShippingRateSource::Static(ShippingRates { dimensional_factor, rates }),
                    })
                })
            }
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
    pub shipping_rates: Option<serde_json::Value>,
    pub dimensional_factor: Option<f64>,
}

impl NewCompaniesPackagesRaw {
    pub fn from_model(new_company_package: NewCompanyPackage) -> Result<Self, FailureError> {
        let NewCompanyPackage {
            company_id,
            package_id,
            shipping_rate_source,
        } = new_company_package;

        match shipping_rate_source.unwrap_or_default() {
            ShippingRateSource::NotAvailable => Ok(NewCompaniesPackagesRaw {
                company_id,

                package_id,
                shipping_rate_source: ShippingRateSourceRaw::NotAvailable,
                shipping_rates: None,
                dimensional_factor: None,
            }),
            ShippingRateSource::Static(ShippingRates { dimensional_factor, rates }) => serde_json::to_value(&rates)
                .map_err(|e| e.context(format!("Failed to serialize {:?}", &rates)).into())
                .map(|rates| NewCompaniesPackagesRaw {
                    company_id,
                    package_id,
                    shipping_rate_source: ShippingRateSourceRaw::Static,
                    shipping_rates: Some(rates),
                    dimensional_factor: Some(dimensional_factor),
                }),
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
