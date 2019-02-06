//! Repo Companies table.

use diesel;
use diesel::connection::AnsiTransactionManager;
use diesel::dsl::sql;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::sql_types::VarChar;
use diesel::Connection;

use errors::Error;
use failure::Error as FailureError;

use stq_types::{Alpha3, CompanyId, UserId};

use models::authorization::*;
use repos::legacy_acl::*;
use repos::types::RepoResult;

use models::companies::{Company, CompanyRaw, NewCompany, UpdateCompany};
use models::countries::Country;
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
    fn find_deliveries_from(&self, country: Alpha3) -> RepoResult<Vec<Company>>;

    /// Update a company
    fn update(&self, id_arg: CompanyId, payload: UpdateCompany) -> RepoResult<Company>;

    /// Delete a company
    fn delete(&self, id_arg: CompanyId) -> RepoResult<Company>;
}

/// Implementation of CompaniesRepo trait
pub struct CompaniesRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<Acl<Resource, Action, Scope, FailureError, Company>>,
    pub countries: Country,
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CompaniesRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<Acl<Resource, Action, Scope, FailureError, Company>>, countries: Country) -> Self {
        Self { db_conn, acl, countries }
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CompaniesRepo for CompaniesRepoImpl<'a, T> {
    fn create(&self, payload: NewCompany) -> RepoResult<Company> {
        debug!("create new company {:?}.", payload);
        let payload = payload.to_raw()?;

        let query = diesel::insert_into(companies).values(&payload);
        query
            .get_result::<CompanyRaw>(self.db_conn)
            .map_err(|e| Error::from(e).into())
            .and_then(|v| Company::from_raw(v, &self.countries))
            .and_then(|company| acl::check(&*self.acl, Resource::Companies, Action::Create, self, Some(&company)).and_then(|_| Ok(company)))
            .map_err(|e: FailureError| e.context(format!("create new company {:?}.", payload)).into())
    }

    fn list(&self) -> RepoResult<Vec<Company>> {
        debug!("List companies");

        let query = companies.order(id);

        query
            .get_results(self.db_conn)
            .map_err(|e| Error::from(e).into())
            .and_then(|raws: Vec<CompanyRaw>| raws.into_iter().map(|v| Company::from_raw(v, &self.countries)).collect())
            .and_then(|results: Vec<Company>| {
                for company in &results {
                    acl::check(&*self.acl, Resource::Companies, Action::Read, self, Some(&company))?;
                }
                Ok(results)
            })
            .map_err(|e: FailureError| e.context("Find in companies error occured").into())
    }

    /// Find specific company by ID
    fn find(&self, id_arg: CompanyId) -> RepoResult<Option<Company>> {
        debug!("Find in company with id {}.", id_arg);

        let query = companies.find(id_arg);
        query
            .get_result::<CompanyRaw>(self.db_conn)
            .optional()
            .map_err(|e| Error::from(e).into())
            .and_then(|company_raw: Option<CompanyRaw>| match company_raw {
                Some(value) => {
                    let company = Company::from_raw(value, &self.countries)?;
                    acl::check(&*self.acl, Resource::Companies, Action::Read, self, Some(&company))?;
                    Ok(Some(company))
                }
                None => Ok(None),
            })
            .map_err(|e: FailureError| e.context(format!("Find company with id: {} error occured", id_arg)).into())
    }

    /// Returns list of companies supported by the country
    fn find_deliveries_from(&self, country: Alpha3) -> RepoResult<Vec<Company>> {
        debug!("Find in companies with country {:?}.", country);

        let query = companies.filter(sql("deliveries_from ? ").bind::<VarChar, _>(&country));

        query
            .get_results(self.db_conn)
            .map_err(|e| Error::from(e).into())
            .and_then(|raw: Vec<CompanyRaw>| raw.into_iter().map(|v| Company::from_raw(v, &self.countries)).collect())
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

        let query = companies.filter(id.eq(id_arg));

        query
            .get_result::<CompanyRaw>(self.db_conn)
            .map_err(|e| Error::from(e).into())
            .and_then(|v| Company::from_raw(v, &self.countries))
            .and_then(|company: Company| acl::check(&*self.acl, Resource::Companies, Action::Update, self, Some(&company)))
            .and_then(|_| {
                let filtered = companies.filter(id.eq(id_arg));

                let query = diesel::update(filtered).set(&payload);
                query
                    .get_result::<CompanyRaw>(self.db_conn)
                    .map_err(|e| Error::from(e).into())
                    .and_then(|v| Company::from_raw(v, &self.countries))
            })
            .map_err(|e: FailureError| e.context(format!("Updating company payload {:?} failed.", payload)).into())
    }

    fn delete(&self, id_arg: CompanyId) -> RepoResult<Company> {
        debug!("delete company by company_id: {}.", id_arg);

        acl::check(&*self.acl, Resource::Companies, Action::Delete, self, None)?;

        let filtered = companies.filter(id.eq(id_arg));
        let query = diesel::delete(filtered);

        query
            .get_result::<CompanyRaw>(self.db_conn)
            .map_err(|e| Error::from(e).into())
            .and_then(|v| Company::from_raw(v, &self.countries))
            .map_err(move |e| e.context(format!("delete company id: {}.", id_arg)).into())
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
