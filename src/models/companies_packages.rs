use stq_types::{CompanyId, CompanyPackageId, CountryLabel, PackageId};

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
pub struct AvailablePackages {
    pub id: CompanyPackageId,
    pub name: String,
    pub deliveries_to: Vec<CountryLabel>,
    pub local_available: bool,
}
