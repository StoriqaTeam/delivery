use models::{Country, Pickups, ShippingVariant};
use stq_static_resources::Currency;
use stq_types::{BaseProductId, CompanyId, CompanyPackageId, PackageId, ProductPrice, ShippingId, StoreId};

use schema::companies_packages;

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, DieselTypes)]
pub enum ShippingRateSource {
    NotAvailable,
    Static,
    OnDemand,
}

impl Default for ShippingRateSource {
    fn default() -> Self {
        ShippingRateSource::NotAvailable
    }
}

#[derive(Serialize, Deserialize, Associations, Queryable, Debug)]
#[table_name = "companies_packages"]
pub struct CompaniesPackages {
    pub id: CompanyPackageId,
    pub company_id: CompanyId,
    pub package_id: PackageId,
    pub shipping_rate_source: ShippingRateSource,
}

#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "companies_packages"]
pub struct NewCompaniesPackages {
    pub company_id: CompanyId,
    pub package_id: PackageId,
    pub shipping_rate_source: Option<ShippingRateSource>,
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
