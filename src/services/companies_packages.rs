//! CompaniesPackages Service, presents CRUD operations
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Fail;
use futures::future::*;
use futures_cpupool::CpuPool;
use r2d2::{ManageConnection, Pool};

use stq_types::{Alpha3, CompanyId, CompanyPackageId, PackageId, UserId};

use errors::Error;
use models::{AvailablePackages, CompaniesPackages, Company, Country, NewCompaniesPackages, Packages};
use repos::countries::{contains_country_code, get_country};
use repos::ReposFactory;
use services::types::ServiceFuture;

pub trait CompaniesPackagesService {
    /// Create a new companies_packages
    fn create(&self, payload: NewCompaniesPackages) -> ServiceFuture<CompaniesPackages>;

    /// Returns available packages supported by the country
    fn find_available_from(&self, country: Alpha3, size: f64, weight: f64) -> ServiceFuture<Vec<AvailablePackages>>;

    /// Returns company package by id
    fn get(&self, id: CompanyPackageId) -> ServiceFuture<CompaniesPackages>;

    /// Returns companies by package id
    fn get_companies(&self, id: PackageId) -> ServiceFuture<Vec<Company>>;

    /// Returns packages by company id
    fn get_packages(&self, id: CompanyId) -> ServiceFuture<Vec<Packages>>;

    /// Delete a companies_packages
    fn delete(&self, id: CompanyPackageId) -> ServiceFuture<CompaniesPackages>;
}

/// CompaniesPackages services, responsible for CRUD operations
pub struct CompaniesPackagesServiceImpl<
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
    > CompaniesPackagesServiceImpl<T, M, F>
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
    > CompaniesPackagesService for CompaniesPackagesServiceImpl<T, M, F>
{
    /// Create a new companies_packages
    fn create(&self, payload: NewCompaniesPackages) -> ServiceFuture<CompaniesPackages> {
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
                            let companies_packages_repo = repo_factory.create_companies_packages_repo(&*conn, user_id);
                            companies_packages_repo.create(payload)
                        })
                })
                .map_err(|e| e.context("Service CompaniesPackages, create endpoint error occured.").into()),
        )
    }

    /// Returns company package by id
    fn get(&self, id: CompanyPackageId) -> ServiceFuture<CompaniesPackages> {
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
                            let companies_packages_repo = repo_factory.create_companies_packages_repo(&*conn, user_id);
                            companies_packages_repo.get(id)
                        })
                })
                .map_err(|e| e.context("Service CompaniesPackages, get endpoint error occured.").into()),
        )
    }

    /// Returns companies by package id
    fn get_companies(&self, id: PackageId) -> ServiceFuture<Vec<Company>> {
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
                            let companies_packages_repo = repo_factory.create_companies_packages_repo(&*conn, user_id);
                            companies_packages_repo.get_companies(id)
                        })
                })
                .map_err(|e| e.context("Service CompaniesPackages, get_companies endpoint error occured.").into()),
        )
    }

    /// Returns packages by company id
    fn get_packages(&self, id: CompanyId) -> ServiceFuture<Vec<Packages>> {
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
                            let companies_packages_repo = repo_factory.create_companies_packages_repo(&*conn, user_id);
                            companies_packages_repo.get_packages(id)
                        })
                })
                .map_err(|e| e.context("Service CompaniesPackages, get_packages endpoint error occured.").into()),
        )
    }

    /// Returns list of companies_packages supported by the country
    fn find_available_from(&self, deliveries_from: Alpha3, size: f64, weight: f64) -> ServiceFuture<Vec<AvailablePackages>> {
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
                            let companies_repo = repo_factory.create_companies_repo(&*conn, user_id);
                            let companies_packages_repo = repo_factory.create_companies_packages_repo(&*conn, user_id);
                            let countries_repo = repo_factory.create_countries_repo(&*conn, user_id);
                            companies_repo
                                .find_deliveries_from(deliveries_from.clone())
                                .map(|companies| companies.into_iter().map(|company| company.id).collect())
                                .and_then(|companies_ids| companies_packages_repo.get_available_packages(companies_ids, size, weight))
                                .and_then({
                                    let deliveries_from = deliveries_from.clone();
                                    move |packages| {
                                        countries_repo.get_all().map(|countries| {
                                            let mut data = vec![];
                                            for package in packages {
                                                let local_available = package.deliveries_to.iter().any(|country_code| {
                                                    get_country(&countries, country_code)
                                                        .map(|c| contains_country_code(&c, &deliveries_from))
                                                        .unwrap_or_default()
                                                });

                                                let deliveries_to = package
                                                    .deliveries_to
                                                    .iter()
                                                    .filter_map(|country_code| get_country(&countries, country_code))
                                                    .collect::<Vec<Country>>();

                                                let element = AvailablePackages {
                                                    id: package.id,
                                                    name: package.name,
                                                    logo: package.logo,
                                                    deliveries_to,
                                                    local_available,
                                                };

                                                data.push(element);
                                            }
                                            data
                                        })
                                    }
                                })
                        })
                })
                .map_err(|e| {
                    e.context("Service CompaniesPackages, find_deliveries_from endpoint error occured.")
                        .into()
                }),
        )
    }

    /// Delete a companies_packages
    fn delete(&self, companies_packages_id: CompanyPackageId) -> ServiceFuture<CompaniesPackages> {
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
                            let companies_packages_repo = repo_factory.create_companies_packages_repo(&*conn, user_id);
                            companies_packages_repo.delete(companies_packages_id)
                        })
                })
                .map_err(|e| e.context("Service CompaniesPackages, delete endpoint error occured.").into()),
        )
    }
}
