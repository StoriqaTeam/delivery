//! Companies Service, presents CRUD operations
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use r2d2::ManageConnection;

use stq_types::{Alpha3, CompanyId};

use models::companies::{Company, NewCompany, UpdateCompany};
use repos::ReposFactory;
use services::types::{Service, ServiceFuture};

pub trait CompaniesService {
    /// Create a new company
    fn create_company(&self, payload: NewCompany) -> ServiceFuture<Company>;

    /// Returns list of companies
    fn list_companies(&self) -> ServiceFuture<Vec<Company>>;

    /// Find specific company by ID
    fn find_company(&self, id: CompanyId) -> ServiceFuture<Option<Company>>;

    /// Returns list of companies supported by the country
    fn find_deliveries_from(&self, country: Alpha3) -> ServiceFuture<Vec<Company>>;

    /// Update a company
    fn update_company(&self, id: CompanyId, payload: UpdateCompany) -> ServiceFuture<Company>;

    /// Delete a company
    fn delete_company(&self, id: CompanyId) -> ServiceFuture<Company>;
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > CompaniesService for Service<T, M, F>
{
    /// Create a new company
    fn create_company(&self, payload: NewCompany) -> ServiceFuture<Company> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let company_repo = repo_factory.create_companies_repo(&*conn, user_id);
            company_repo
                .create(payload)
                .map_err(|e| e.context("Service Companies, create endpoint error occured.").into())
        })
    }

    /// Returns list of companies
    fn list_companies(&self) -> ServiceFuture<Vec<Company>> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let company_repo = repo_factory.create_companies_repo(&*conn, user_id);
            company_repo
                .list()
                .map_err(|e| e.context("Service Companies, list endpoint error occured.").into())
        })
    }

    /// Find specific company by ID
    fn find_company(&self, company_id: CompanyId) -> ServiceFuture<Option<Company>> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let company_repo = repo_factory.create_companies_repo(&*conn, user_id);
            company_repo
                .find(company_id)
                .map_err(|e| e.context("Service Companies, find endpoint error occured.").into())
        })
    }

    /// Returns list of companies supported by the country
    fn find_deliveries_from(&self, country: Alpha3) -> ServiceFuture<Vec<Company>> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let company_repo = repo_factory.create_companies_repo(&*conn, user_id);
            company_repo
                .find_deliveries_from(country)
                .map_err(|e| e.context("Service Companies, find_deliveries_from endpoint error occured.").into())
        })
    }

    /// Update a company
    fn update_company(&self, id: CompanyId, payload: UpdateCompany) -> ServiceFuture<Company> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let company_repo = repo_factory.create_companies_repo(&*conn, user_id);
            company_repo
                .update(id, payload)
                .map_err(|e| e.context("Service Companies, update endpoint error occured.").into())
        })
    }

    /// Delete a company
    fn delete_company(&self, company_id: CompanyId) -> ServiceFuture<Company> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let company_repo = repo_factory.create_companies_repo(&*conn, user_id);
            company_repo
                .delete(company_id)
                .map_err(|e| e.context("Service Companies, delete endpoint error occured.").into())
        })
    }
}
