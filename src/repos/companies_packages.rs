//! Repo companies_packages table.

use diesel;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::Connection;

use errors::Error;
use failure::Error as FailureError;
use failure::Fail;

use stq_types::{CompanyId, CompanyPackageId, PackageId, UserId};

use models::authorization::*;
use repos::legacy_acl::*;
use repos::types::RepoResult;

use extras::option::transpose;
use models::{
    get_country, AvailablePackages, CompaniesPackagesRaw, Company, CompanyPackage, CompanyRaw, Country, NewCompaniesPackagesRaw,
    NewCompanyPackage, Packages, PackagesRaw,
};
use repos::*;
use schema::companies::dsl as DslCompanies;
use schema::companies_packages::dsl::*;
use schema::packages::dsl as DslPackages;

/// Companies packages repository for handling companies_packages model
pub trait CompaniesPackagesRepo {
    /// Create a new companies_packages
    fn create(&self, payload: NewCompanyPackage) -> RepoResult<CompanyPackage>;

    /// Getting available packages satisfying the constraints
    fn get_available_packages(
        &self,
        company_id_args: Vec<CompanyId>,
        size: u32,
        weight: u32,
        deliveries_from: Alpha3,
    ) -> RepoResult<Vec<AvailablePackages>>;

    /// Returns company package by id
    fn get(&self, id: CompanyPackageId) -> RepoResult<Option<CompanyPackage>>;

    /// Returns companies by package id
    fn get_companies(&self, id: PackageId) -> RepoResult<Vec<Company>>;

    /// Returns packages by company id
    fn get_packages(&self, id: CompanyId) -> RepoResult<Vec<Packages>>;

    /// Delete a companies_packages
    fn delete(&self, company_id_arg: CompanyId, package_id_arg: PackageId) -> RepoResult<CompanyPackage>;
}

/// Implementation of CompaniesPackagesRepo trait
pub struct CompaniesPackagesRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<Acl<Resource, Action, Scope, FailureError, CompanyPackage>>,
    pub countries: Country,
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CompaniesPackagesRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<Acl<Resource, Action, Scope, FailureError, CompanyPackage>>, countries: Country) -> Self {
        Self { db_conn, acl, countries }
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CompaniesPackagesRepo
    for CompaniesPackagesRepoImpl<'a, T>
{
    fn create(&self, payload: NewCompanyPackage) -> RepoResult<CompanyPackage> {
        debug!("create new companies_packages {:?}.", payload);
        let record = NewCompaniesPackagesRaw::from(payload.clone());

        let query = diesel::insert_into(companies_packages).values(&record);
        query
            .get_result::<CompaniesPackagesRaw>(self.db_conn)
            .map_err(|e| Error::from(e).into())
            .and_then(CompaniesPackagesRaw::to_model)
            .and_then(|company_package| {
                acl::check(
                    &*self.acl,
                    Resource::CompaniesPackages,
                    Action::Create,
                    self,
                    Some(&company_package),
                )?;
                Ok(company_package)
            })
            .map_err(|e: FailureError| e.context(format!("create new companies_packages {:?}.", payload)).into())
    }

    fn get(&self, id_arg: CompanyPackageId) -> RepoResult<Option<CompanyPackage>> {
        debug!("get companies_packages by id: {}.", id_arg);

        acl::check(&*self.acl, Resource::CompaniesPackages, Action::Read, self, None)?;
        let query = companies_packages.filter(id.eq(id_arg));
        query
            .get_result::<CompaniesPackagesRaw>(self.db_conn)
            .optional()
            .map_err(move |e| Error::from(e).context(format!("get companies_packages id: {}.", id_arg)).into())
            .and_then(|record| transpose(record.map(CompaniesPackagesRaw::to_model)))
    }

    /// Getting available packages satisfying the constraints
    fn get_available_packages(
        &self,
        company_id_args: Vec<CompanyId>,
        size: u32,
        weight: u32,
        deliveries_from: Alpha3,
    ) -> RepoResult<Vec<AvailablePackages>> {
        let size = size as i32;
        let weight = weight as i32;

        debug!(
            "Find in packages with companies: {:?}, size: {}, weight: {}.",
            company_id_args, size, weight
        );

        let query = companies_packages
            .filter(company_id.eq_any(&company_id_args))
            .inner_join(DslCompanies::companies)
            .inner_join(DslPackages::packages)
            .filter(DslPackages::max_size.ge(size))
            .filter(DslPackages::min_size.le(size))
            .filter(DslPackages::max_weight.ge(weight))
            .filter(DslPackages::min_weight.le(weight))
            .order(DslCompanies::label);

        query
            .get_results::<(CompaniesPackagesRaw, CompanyRaw, PackagesRaw)>(self.db_conn)
            .map_err(|e| Error::from(e).into())
            .and_then(|results| {
                let mut data = vec![];

                for result in results {
                    let (companies_package, company_raw, package_raw) = result;
                    let company_package = companies_package.to_model()?;
                    let used_codes = package_raw.get_deliveries_to()?;

                    let local_available = used_codes.iter().any(|country_code| {
                        get_country(&self.countries, country_code)
                            .map(|c| contains_country_code(&c, &deliveries_from))
                            .unwrap_or_default()
                    });

                    let package = package_raw.to_packages(&self.countries)?;

                    data.push(AvailablePackages {
                        id: company_package.id,
                        name: get_company_package_name(&company_raw.label, &package.name),
                        logo: company_raw.logo,
                        deliveries_to: package.deliveries_to,
                        shipping_rate_source: company_package.shipping_rate_source,
                        currency: company_raw.currency,
                        local_available,
                    });
                }

                Ok(data)
            })
            .map_err(|e: FailureError| {
                e.context(format!(
                    "Find in packages with  companies: {:?}, size: {}, weight: {} error occured",
                    company_id_args, size, weight
                ))
                .into()
            })
    }

    /// Returns companies by package id
    fn get_companies(&self, id_arg: PackageId) -> RepoResult<Vec<Company>> {
        debug!("get companies_packages by package_id: {}.", id_arg);

        let query = companies_packages.filter(package_id.eq(id_arg)).inner_join(DslCompanies::companies);

        query
            .get_results::<(CompaniesPackagesRaw, CompanyRaw)>(self.db_conn)
            .map_err(|e| Error::from(e).into())
            .and_then(|results| {
                let mut data = vec![];
                for result in results {
                    let (_, company_raw) = result;
                    let element = Company::from_raw(company_raw, &self.countries)?;
                    data.push(element);
                }

                Ok(data)
            })
            .map_err(move |e: FailureError| e.context(format!("get companies_packages package_id: {}.", id_arg)).into())
    }

    /// Returns packages by company id
    fn get_packages(&self, id_arg: CompanyId) -> RepoResult<Vec<Packages>> {
        debug!("get companies_packages by company_id: {}.", id_arg);

        let query = companies_packages.filter(company_id.eq(id_arg)).inner_join(DslPackages::packages);

        query
            .get_results::<(CompaniesPackagesRaw, PackagesRaw)>(self.db_conn)
            .map_err(|e| Error::from(e).into())
            .and_then(|results| {
                let mut data = vec![];
                for result in results {
                    let (_, package_raw) = result;
                    let element = package_raw.to_packages(&self.countries)?;
                    data.push(element);
                }

                Ok(data)
            })
            .map_err(move |e: FailureError| e.context(format!("get companies_packages company_id: {}.", id_arg)).into())
    }

    fn delete(&self, company_id_arg: CompanyId, package_id_arg: PackageId) -> RepoResult<CompanyPackage> {
        debug!(
            "delete companies_packages by company_id: {}, package_id: {}.",
            company_id_arg, package_id_arg
        );

        acl::check(&*self.acl, Resource::CompaniesPackages, Action::Delete, self, None)?;
        let filtered = companies_packages.filter(company_id.eq(company_id_arg).and(package_id.eq(package_id_arg)));
        let query = diesel::delete(filtered);
        query
            .get_result::<CompaniesPackagesRaw>(self.db_conn)
            .map_err(move |e| {
                Error::from(e)
                    .context(format!(
                        "delete companies_packages company_id: {}, package_id: {}.",
                        company_id_arg, package_id_arg
                    ))
                    .into()
            })
            .and_then(CompaniesPackagesRaw::to_model)
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, CompanyPackage>
    for CompaniesPackagesRepoImpl<'a, T>
{
    fn is_in_scope(&self, _user_id: UserId, scope: &Scope, _obj: Option<&CompanyPackage>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => false,
        }
    }
}
