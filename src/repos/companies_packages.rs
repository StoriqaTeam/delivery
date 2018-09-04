//! Repo companies_packages table.

use diesel;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::Connection;

use failure::Error as FailureError;
use failure::Fail;

use stq_types::{CompanyId, CompanyPackageId, UserId};

use models::authorization::*;
use repos::legacy_acl::*;
use repos::types::RepoResult;

use models::{CompaniesPackages, CompanyRaw, InnerAvailablePackages, NewCompaniesPackages, PackagesRaw};
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
            .and_then(|record| acl::check(&*self.acl, Resource::Companies, Action::Create, self, Some(&record)).and_then(|_| Ok(record)))
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
            .inner_join(DslCompanies::companies)
            .inner_join(DslPackages::packages)
            .filter(company_id.eq_any(&company_id_args))
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
                        name: format!("{}-{}", company_raw.label, package.name),
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
