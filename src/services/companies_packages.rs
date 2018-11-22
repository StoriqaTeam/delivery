//! CompaniesPackages Service, presents CRUD operations
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;

use r2d2::ManageConnection;

use failure::Error as FailureError;

use stq_types::{Alpha3, CompanyId, CompanyPackageId, PackageId};

use models::{AvailablePackages, Company, CompanyPackage, NewCompanyPackage, Packages};
use repos::ReposFactory;
use services::types::{Service, ServiceFuture};

pub trait CompaniesPackagesService {
    /// Create a new companies_packages
    fn create_company_package(&self, payload: NewCompanyPackage) -> ServiceFuture<CompanyPackage>;

    /// Returns available packages supported by the country
    fn get_available_packages(&self, country: Alpha3, size: f64, weight: f64) -> ServiceFuture<Vec<AvailablePackages>>;

    /// Returns company package by id
    fn get_company_package(&self, id: CompanyPackageId) -> ServiceFuture<Option<CompanyPackage>>;

    /// Returns companies by package id
    fn get_companies(&self, id: PackageId) -> ServiceFuture<Vec<Company>>;

    /// Returns packages by company id
    fn get_packages(&self, id: CompanyId) -> ServiceFuture<Vec<Packages>>;

    /// Delete a companies_packages
    fn delete_company_package(&self, company_id: CompanyId, package_id: PackageId) -> ServiceFuture<CompanyPackage>;
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > CompaniesPackagesService for Service<T, M, F>
{
    /// Create a new companies_packages
    fn create_company_package(&self, payload: NewCompanyPackage) -> ServiceFuture<CompanyPackage> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let companies_packages_repo = repo_factory.create_companies_packages_repo(&*conn, user_id);
            conn.transaction::<CompanyPackage, FailureError, _>(move || {
                companies_packages_repo
                    .create(payload)
                    .map_err(|e| e.context("Service CompaniesPackages, create endpoint error occured.").into())
            })
        })
    }

    /// Returns company package by id
    fn get_company_package(&self, id: CompanyPackageId) -> ServiceFuture<Option<CompanyPackage>> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let companies_packages_repo = repo_factory.create_companies_packages_repo(&*conn, user_id);
            companies_packages_repo
                .get(id)
                .map_err(|e| e.context("Service CompaniesPackages, get endpoint error occured.").into())
        })
    }

    /// Returns companies by package id
    fn get_companies(&self, id: PackageId) -> ServiceFuture<Vec<Company>> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let companies_packages_repo = repo_factory.create_companies_packages_repo(&*conn, user_id);
            companies_packages_repo
                .get_companies(id)
                .map_err(|e| e.context("Service CompaniesPackages, get_companies endpoint error occured.").into())
        })
    }

    /// Returns packages by company id
    fn get_packages(&self, id: CompanyId) -> ServiceFuture<Vec<Packages>> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let companies_packages_repo = repo_factory.create_companies_packages_repo(&*conn, user_id);
            companies_packages_repo
                .get_packages(id)
                .map_err(|e| e.context("Service CompaniesPackages, get_packages endpoint error occured.").into())
        })
    }

    /// Returns list of companies_packages supported by the country
    fn get_available_packages(&self, deliveries_from: Alpha3, size: f64, weight: f64) -> ServiceFuture<Vec<AvailablePackages>> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let companies_repo = repo_factory.create_companies_repo(&*conn, user_id);
            let companies_packages_repo = repo_factory.create_companies_packages_repo(&*conn, user_id);
            companies_repo
                .find_deliveries_from(deliveries_from.clone())
                .map(|companies| companies.into_iter().map(|company| company.id).collect())
                .and_then(|companies_ids| {
                    companies_packages_repo.get_available_packages(companies_ids, size, weight, deliveries_from.clone())
                }).map_err(|e| {
                    e.context("Service CompaniesPackages, find_deliveries_from endpoint error occured.")
                        .into()
                })
        })
    }

    /// Delete a companies_packages
    fn delete_company_package(&self, company_id: CompanyId, package_id: PackageId) -> ServiceFuture<CompanyPackage> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let companies_packages_repo = repo_factory.create_companies_packages_repo(&*conn, user_id);
            companies_packages_repo
                .delete(company_id, package_id)
                .map_err(|e| e.context("Service CompaniesPackages, delete endpoint error occured.").into())
        })
    }
}
