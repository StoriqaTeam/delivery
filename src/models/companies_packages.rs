use models::{Country, Pickups};
use stq_types::{Alpha3, CompanyId, CompanyPackageId, PackageId, ProductPrice};

use schema::companies_packages;

#[derive(Serialize, Deserialize, Associations, Queryable, Debug)]
#[table_name = "companies_packages"]
pub struct CompaniesPackages {
    pub id: CompanyPackageId,
    pub company_id: CompanyId,
    pub package_id: PackageId,
}

#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "companies_packages"]
pub struct NewCompaniesPackages {
    pub company_id: CompanyId,
    pub package_id: PackageId,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InnerAvailablePackages {
    pub id: CompanyPackageId,
    pub name: String,
    pub logo: String,
    pub deliveries_to: Vec<Alpha3>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AvailablePackages {
    pub id: CompanyPackageId,
    pub name: String,
    pub logo: String,
    pub deliveries_to: Vec<Country>,
    pub local_available: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AvailablePackageForUser {
    pub id: CompanyPackageId,
    pub name: String,
    pub logo: String,
    pub price: Option<ProductPrice>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AvailableShipppingForUser {
    pub packages: Vec<AvailablePackageForUser>,
    pub pickups: Option<Pickups>,
}
