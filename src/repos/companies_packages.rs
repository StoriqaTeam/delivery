//! Repo companies_packages table.

use diesel;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::Connection;

use serde_json;

use failure::Error as FailureError;
use failure::Fail;

use stq_types::UserId;

use models::authorization::*;
use repos::legacy_acl::*;
use repos::types::RepoResult;

use errors::Error;
use models::companies_packages::{AvailablePackages, CompaniesPackages, NewCompaniesPackages};
use repos::*;
use schema::companies::dsl as DslCompanies;
use schema::companies_packages::dsl::*;
use schema::packages::dsl as DslPackages;

/// Companies packages repository for handling companies_packages model
pub trait CompaniesPackagesRepo {
    /// Create a new company
    fn create(&self, payload: NewCompaniesPackages) -> RepoResult<CompaniesPackages>;

    /// Getting available packages satisfying the constraints
    fn get_available_packages(&self, company_id_args: Vec<i32>, size: f64, weight: f64) -> RepoResult<Vec<AvailablePackages>>;

    /// Delete a companies_packages
    fn delete(&self, id_arg: i32) -> RepoResult<CompaniesPackages>;
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

    /// Getting available packages satisfying the constraints
    fn get_available_packages(&self, company_id_args: Vec<i32>, size: f64, weight: f64) -> RepoResult<Vec<AvailablePackages>> {
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
            .select((id, DslCompanies::label, DslPackages::name, DslPackages::deliveries_to));

        query
            .get_results(self.db_conn)
            .map_err(From::from)
            .and_then(|results: Vec<(i32, String, String, serde_json::Value)>| {
                let mut data = vec![];
                for result in results {
                    let deliveries_to = serde_json::from_value(result.3)
                        .map_err(|e| e.context("Can not parse deliveries_to from db").context(Error::Parse))?;
                    data.push(AvailablePackages {
                        id: result.0,
                        name: format!("{}-{}", result.1, result.2),
                        deliveries_to,
                    });
                }
                //acl::check(&*self.acl, Resource::DeliveryTo, Action::Read, self, Some(&delivery))?;
                Ok(data)
            })
            .map_err(|e: FailureError| {
                e.context(format!(
                    "Find in packages with  companies: {:?}, size: {}, weight: {} error occured",
                    company_id_args, size, weight
                )).into()
            })
    }

    fn delete(&self, id_arg: i32) -> RepoResult<CompaniesPackages> {
        debug!("delete companies_packages by id: {}.", id_arg);

        acl::check(&*self.acl, Resource::CompaniesPackages, Action::Delete, self, None)?;
        let filtered = companies_packages.filter(id.eq(id_arg.clone()));
        let query = diesel::delete(filtered);
        query
            .get_result(self.db_conn)
            .map_err(move |e| e.context(format!("delete company id: {}.", id_arg)).into())
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
