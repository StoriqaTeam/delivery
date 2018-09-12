//! Repo companies_packages table.

use diesel;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::Connection;

use failure::Error as FailureError;
use failure::Fail;

use stq_types::{CompanyId, CompanyPackageId, PackageId, UserId};

use models::authorization::*;
use repos::legacy_acl::*;
use repos::types::RepoResult;

use models::{CompaniesPackages, Company, CompanyRaw, InnerAvailablePackages, NewCompaniesPackages, Packages, PackagesRaw};
use repos::*;
use schema::companies::dsl as DslCompanies;
use schema::companies_packages::dsl::*;
use schema::packages::dsl as DslPackages;

/// Companies packages repository for handling companies_packages model
pub trait CompaniesPackagesRepo {
    /// Create a new companies_packages
    fn create(&self, payload: NewCompaniesPackages) -> RepoResult<CompaniesPackages>;

    /// Getting available packages satisfying the constraints
    fn get_available_packages(&self, company_id_args: Vec<CompanyId>, size: f64, weight: f64) -> RepoResult<Vec<InnerAvailablePackages>>;

    /// Returns company package by id
    fn get(&self, id: CompanyPackageId) -> RepoResult<CompaniesPackages>;

    /// Returns companies by package id
    fn get_companies(&self, id: PackageId) -> RepoResult<Vec<Company>>;

    /// Returns packages by company id
    fn get_packages(&self, id: CompanyId) -> RepoResult<Vec<Packages>>;

    /// Delete a companies_packages
    fn delete(&self, id_arg: CompanyPackageId) -> RepoResult<CompaniesPackages>;
}

/// Implementation of CompaniesPackagesRepo trait
pub struct CompaniesPackagesRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<Acl<Resource, Action, Scope, FailureError, CompaniesPackages>>,
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CompaniesPackagesRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<Acl<Resource, Action, Scope, FailureError, CompaniesPackages>>) -> Self {
        Self { db_conn, acl }
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CompaniesPackagesRepo
    for CompaniesPackagesRepoImpl<'a, T>
{
    fn create(&self, payload: NewCompaniesPackages) -> RepoResult<CompaniesPackages> {
        debug!("create new companies_packages {:?}.", payload);
        let query = diesel::insert_into(companies_packages).values(&payload);
        query
            .get_result::<CompaniesPackages>(self.db_conn)
            .map_err(From::from)
            .and_then(|record| {
                acl::check(&*self.acl, Resource::CompaniesPackages, Action::Create, self, Some(&record)).and_then(|_| Ok(record))
            })
            .map_err(|e: FailureError| e.context(format!("create new companies_packages {:?}.", payload)).into())
    }

    fn get(&self, id_arg: CompanyPackageId) -> RepoResult<CompaniesPackages> {
        debug!("get companies_packages by id: {}.", id_arg);

        acl::check(&*self.acl, Resource::CompaniesPackages, Action::Read, self, None)?;
        let query = companies_packages.filter(id.eq(id_arg));
        query
            .get_result(self.db_conn)
            .map_err(move |e| e.context(format!("get companies_packages id: {}.", id_arg)).into())
    }

    /// Getting available packages satisfying the constraints
    fn get_available_packages(&self, company_id_args: Vec<CompanyId>, size: f64, weight: f64) -> RepoResult<Vec<InnerAvailablePackages>> {
        debug!(
            "Find in packages with companies: {:?}, size: {}, weight: {}.",
            company_id_args, size, weight
        );

        let query = companies_packages
            .filter(company_id.eq_any(&company_id_args))
            .inner_join(DslCompanies::companies)
            .inner_join(DslPackages::packages)
            .filter(DslPackages::max_size.le(size))
            .filter(DslPackages::min_size.ge(size))
            .filter(DslPackages::max_weight.le(size))
            .filter(DslPackages::min_weight.ge(size))
            .order(DslCompanies::label);

        query
            .get_results::<(CompaniesPackages, CompanyRaw, PackagesRaw)>(self.db_conn)
            .map_err(From::from)
            .and_then(|results| {
                let mut data = vec![];
                for result in results {
                    let (companies_package, company_raw, package_raw) = result;
                    let package = package_raw.to_packages()?;
                    data.push(InnerAvailablePackages {
                        id: companies_package.id,
                        name: get_company_package_name(company_raw.label, package.name),
                        logo: company_raw.logo,
                        deliveries_to: package.deliveries_to,
                    });
                }

                Ok(data)
            })
            .map_err(|e: FailureError| {
                e.context(format!(
                    "Find in packages with  companies: {:?}, size: {}, weight: {} error occured",
                    company_id_args, size, weight
                )).into()
            })
    }

    /// Returns companies by package id
    fn get_companies(&self, id_arg: PackageId) -> RepoResult<Vec<Company>> {
        debug!("get companies_packages by package_id: {}.", id_arg);

        let query = companies_packages.filter(package_id.eq(id_arg)).inner_join(DslCompanies::companies);

        query
            .get_results::<(CompaniesPackages, CompanyRaw)>(self.db_conn)
            .map_err(From::from)
            .and_then(|results| {
                let mut data = vec![];
                for result in results {
                    let (_, company_raw) = result;
                    let element = Company::from_raw(company_raw)?;
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
            .get_results::<(CompaniesPackages, PackagesRaw)>(self.db_conn)
            .map_err(From::from)
            .and_then(|results| {
                let mut data = vec![];
                for result in results {
                    let (_, package_raw) = result;
                    let element = package_raw.to_packages()?;
                    data.push(element);
                }

                Ok(data)
            })
            .map_err(move |e: FailureError| e.context(format!("get companies_packages company_id: {}.", id_arg)).into())
    }

    fn delete(&self, id_arg: CompanyPackageId) -> RepoResult<CompaniesPackages> {
        debug!("delete companies_packages by id: {}.", id_arg);

        acl::check(&*self.acl, Resource::CompaniesPackages, Action::Delete, self, None)?;
        let filtered = companies_packages.filter(id.eq(id_arg));
        let query = diesel::delete(filtered);
        query
            .get_result(self.db_conn)
            .map_err(move |e| e.context(format!("delete companies_packages id: {}.", id_arg)).into())
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, CompaniesPackages>
    for CompaniesPackagesRepoImpl<'a, T>
{
    fn is_in_scope(&self, _user_id: UserId, scope: &Scope, _obj: Option<&CompaniesPackages>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => false,
        }
    }
}
