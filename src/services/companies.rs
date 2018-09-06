//! Companies Service, presents CRUD operations
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Fail;
use futures::future::*;
use futures_cpupool::CpuPool;
use r2d2::{ManageConnection, Pool};

use stq_types::{Alpha3, CompanyId, UserId};

use errors::Error;
use models::companies::{Company, NewCompany, UpdateCompany};
use repos::ReposFactory;
use services::types::ServiceFuture;

pub trait CompaniesService {
    /// Create a new company
    fn create(&self, payload: NewCompany) -> ServiceFuture<Company>;

    /// Returns list of companies
    fn list(&self) -> ServiceFuture<Vec<Company>>;

    /// Find specific company by ID
    fn find(&self, id: CompanyId) -> ServiceFuture<Option<Company>>;

    /// Returns list of companies supported by the country
    fn find_deliveries_from(&self, country: Alpha3) -> ServiceFuture<Vec<Company>>;

    /// Update a company
    fn update(&self, id: CompanyId, payload: UpdateCompany) -> ServiceFuture<Company>;

    /// Delete a company
    fn delete(&self, id: CompanyId) -> ServiceFuture<Company>;
}

/// Companies services, responsible for CRUD operations
pub struct CompaniesServiceImpl<
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    M: ManageConnection<Connection = T>,
    F: ReposFactory<T>,
> {
    pub db_pool: Pool<M>,
    pub cpu_pool: CpuPool,
    pub user_id: Option<UserId>,
    pub repo_factory: F,
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > CompaniesServiceImpl<T, M, F>
{
    pub fn new(db_pool: Pool<M>, cpu_pool: CpuPool, user_id: Option<UserId>, repo_factory: F) -> Self {
        Self {
            db_pool,
            cpu_pool,
            user_id,
            repo_factory,
        }
    }
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > CompaniesService for CompaniesServiceImpl<T, M, F>
{
    /// Create a new company
    fn create(&self, payload: NewCompany) -> ServiceFuture<Company> {
        let db_pool = self.db_pool.clone();
        let repo_factory = self.repo_factory.clone();
        let user_id = self.user_id;

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(Error::Connection).into())
                        .and_then(move |conn| {
                            let company_repo = repo_factory.create_companies_repo(&*conn, user_id);
                            company_repo.create(payload)
                        })
                })
                .map_err(|e| e.context("Service Companies, create endpoint error occured.").into()),
        )
    }

    /// Returns list of companies
    fn list(&self) -> ServiceFuture<Vec<Company>> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let repo_factory = self.repo_factory.clone();

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(Error::Connection).into())
                        .and_then(move |conn| {
                            let company_repo = repo_factory.create_companies_repo(&*conn, user_id);
                            company_repo.list()
                        })
                })
                .map_err(|e| e.context("Service Companies, list endpoint error occured.").into()),
        )
    }

    /// Find specific company by ID
    fn find(&self, company_id: CompanyId) -> ServiceFuture<Option<Company>> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let repo_factory = self.repo_factory.clone();

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(Error::Connection).into())
                        .and_then(move |conn| {
                            let company_repo = repo_factory.create_companies_repo(&*conn, user_id);
                            company_repo.find(company_id)
                        })
                })
                .map_err(|e| e.context("Service Companies, find endpoint error occured.").into()),
        )
    }

    /// Returns list of companies supported by the country
    fn find_deliveries_from(&self, country: Alpha3) -> ServiceFuture<Vec<Company>> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let repo_factory = self.repo_factory.clone();

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(Error::Connection).into())
                        .and_then(move |conn| {
                            let company_repo = repo_factory.create_companies_repo(&*conn, user_id);
                            company_repo.find_deliveries_from(country)
                        })
                })
                .map_err(|e| e.context("Service Companies, find_deliveries_from endpoint error occured.").into()),
        )
    }

    /// Update a company
    fn update(&self, id: CompanyId, payload: UpdateCompany) -> ServiceFuture<Company> {
        let db_pool = self.db_pool.clone();
        let repo_factory = self.repo_factory.clone();
        let user_id = self.user_id;

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(Error::Connection).into())
                        .and_then(move |conn| {
                            let company_repo = repo_factory.create_companies_repo(&*conn, user_id);
                            company_repo.update(id, payload)
                        })
                })
                .map_err(|e| e.context("Service Companies, update endpoint error occured.").into()),
        )
    }

    /// Delete a company
    fn delete(&self, company_id: CompanyId) -> ServiceFuture<Company> {
        let db_pool = self.db_pool.clone();
        let repo_factory = self.repo_factory.clone();
        let user_id = self.user_id;

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(Error::Connection).into())
                        .and_then(move |conn| {
                            let company_repo = repo_factory.create_companies_repo(&*conn, user_id);
                            company_repo.delete(company_id)
                        })
                })
                .map_err(|e| e.context("Service Companies, delete endpoint error occured.").into()),
        )
    }
}
