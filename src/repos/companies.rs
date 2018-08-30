//! Repo Companies table.

use diesel;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::Connection;

use failure::Error as FailureError;
use failure::Fail;

use stq_types::{CompanyId, CountryLabel, UserId};

use models::authorization::*;
use repos::legacy_acl::*;
use repos::types::RepoResult;

use models::companies::{Company, CompanyRaw, NewCompany, UpdateCompany};
use repos::*;
use schema::companies::dsl::*;

/// Companies repository for handling Companies
pub trait CompaniesRepo {
    /// Create a new company
    fn create(&self, payload: NewCompany) -> RepoResult<Company>;

    /// Returns list of companies
    fn list(&self) -> RepoResult<Vec<Company>>;

    /// Find specific company by ID
    fn find(&self, id_arg: CompanyId) -> RepoResult<Option<Company>>;

    /// Returns list of companies supported by the country
    fn find_deliveries_from(&self, country: CountryLabel) -> RepoResult<Vec<Company>>;

    /// Update a company
    fn update(&self, id_arg: CompanyId, payload: UpdateCompany) -> RepoResult<Company>;

    /// Delete a company
    fn delete(&self, id_arg: CompanyId) -> RepoResult<Company>;
}

/// Implementation of CompaniesRepo trait
pub struct CompaniesRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<Acl<Resource, Action, Scope, FailureError, Company>>,
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CompaniesRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<Acl<Resource, Action, Scope, FailureError, Company>>) -> Self {
        Self { db_conn, acl }
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CompaniesRepo for CompaniesRepoImpl<'a, T> {
    fn create(&self, payload: NewCompany) -> RepoResult<Company> {
        debug!("create new company {:?}.", payload);
        let payload = payload.to_raw()?;
        let query = diesel::insert_into(companies).values(&payload);
        query
            .get_result::<CompanyRaw>(self.db_conn)
            .map_err(From::from)
            .and_then(Company::from_raw)
            .and_then(|company| acl::check(&*self.acl, Resource::Companies, Action::Create, self, Some(&company)).and_then(|_| Ok(company)))
            .map_err(|e: FailureError| e.context(format!("create new company {:?}.", payload)).into())
    }

    fn list(&self) -> RepoResult<Vec<Company>> {
        debug!("List companies");
        let query = companies.order(id);

        query
            .get_results(self.db_conn)
            .map_err(From::from)
            .and_then(|raws: Vec<CompanyRaw>| raws.into_iter().map(Company::from_raw).collect())
            .and_then(|results: Vec<Company>| {
                for company in &results {
                    acl::check(&*self.acl, Resource::Companies, Action::Read, self, Some(&company))?;
                }
                Ok(results)
            })
            .map_err(|e: FailureError| e.context(format!("Find in companies error occured")).into())
    }

    /// Find specific company by ID
    fn find(&self, id_arg: CompanyId) -> RepoResult<Option<Company>> {
        debug!("Find in company with id {}.", id_arg);
        let query = companies.find(id_arg);
        query
            .get_result::<CompanyRaw>(self.db_conn)
            .optional()
            .map_err(From::from)
            .and_then(|company_raw: Option<CompanyRaw>| match company_raw {
                Some(value) => {
                    let company = Company::from_raw(value)?;
                    acl::check(&*self.acl, Resource::Companies, Action::Read, self, Some(&company))?;
                    Ok(Some(company))
                }
                None => Ok(None),
            })
            .map_err(|e: FailureError| e.context(format!("Find company with id: {} error occured", id_arg)).into())
    }

    /// Returns list of companies supported by the country
    fn find_deliveries_from(&self, country: CountryLabel) -> RepoResult<Vec<Company>> {
        debug!("Find in companies with country {:?}.", country);

        let query_str = format!("SELECT * FROM companies WHERE deliveries_from @> {};", country);
        diesel::sql_query(query_str)
            .get_results(self.db_conn)
            .map_err(From::from)
            .and_then(|raw: Vec<CompanyRaw>| raw.into_iter().map(Company::from_raw).collect())
            .and_then(|results: Vec<Company>| {
                for result in &results {
                    acl::check(&*self.acl, Resource::Companies, Action::Read, self, Some(&result))?;
                }
                Ok(results)
            })
            .map_err(|e: FailureError| {
                e.context(format!("Find in companies with country {:?} error occured", country))
                    .into()
            })
    }

    fn update(&self, id_arg: CompanyId, payload: UpdateCompany) -> RepoResult<Company> {
        debug!("Updating company {} with payload {:?}.", id_arg, payload);
        let payload = payload.to_raw()?;
        let query = companies.filter(id.eq(id_arg.clone()));

        query
            .get_result::<CompanyRaw>(self.db_conn)
            .map_err(From::from)
            .and_then(Company::from_raw)
            .and_then(|company: Company| acl::check(&*self.acl, Resource::Companies, Action::Update, self, Some(&company)))
            .and_then(|_| {
                let filtered = companies.filter(id.eq(id_arg.clone()));

                let query = diesel::update(filtered).set(&payload);
                query
                    .get_result::<CompanyRaw>(self.db_conn)
                    .map_err(From::from)
                    .and_then(Company::from_raw)
            })
            .map_err(|e: FailureError| e.context(format!("Updating company payload {:?} failed.", payload)).into())
    }

    fn delete(&self, id_arg: CompanyId) -> RepoResult<Company> {
        debug!("delete company by company_id: {}.", id_arg);

        acl::check(&*self.acl, Resource::Companies, Action::Delete, self, None)?;
        let filtered = companies.filter(id.eq(id_arg.clone()));
        let query = diesel::delete(filtered);
        query
            .get_result::<CompanyRaw>(self.db_conn)
            .map_err(move |e| e.context(format!("delete company id: {}.", id_arg)).into())
            .and_then(Company::from_raw)
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, Company>
    for CompaniesRepoImpl<'a, T>
{
    fn is_in_scope(&self, _user_id: UserId, scope: &Scope, _obj: Option<&Company>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => false,
        }
    }
}
