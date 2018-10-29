//! Countries Services, presents CRUD operations with countries

use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Error as FailureError;
use r2d2::ManageConnection;

use stq_types::Alpha3;

use super::types::{Service, ServiceFuture};
use models::{Country, NewCountry};
use repos::{CountrySearch, ReposFactory};

pub trait CountriesService {
    /// Creates new country
    fn create_country(&self, payload: NewCountry) -> ServiceFuture<Country>;
    /// Returns country by code
    fn get_country(&self, label: Alpha3) -> ServiceFuture<Option<Country>>;
    /// Returns country by codes
    fn find_country(&self, search: CountrySearch) -> ServiceFuture<Option<Country>>;
    /// Returns all countries as a tree
    fn get_all(&self) -> ServiceFuture<Country>;
    /// Returns all countries as a flat Vec
    fn get_all_flatten(&self) -> ServiceFuture<Vec<Country>>;
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > CountriesService for Service<T, M, F>
{
    /// Returns country by code
    fn get_country(&self, code: Alpha3) -> ServiceFuture<Option<Country>> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let countries_repo = repo_factory.create_countries_repo(&*conn, user_id);
            countries_repo
                .find(code)
                .map_err(|e| e.context("Service Countries, get endpoint error occured.").into())
        })
    }

    /// Returns country by codes
    fn find_country(&self, search: CountrySearch) -> ServiceFuture<Option<Country>> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let countries_repo = repo_factory.create_countries_repo(&*conn, user_id);
            countries_repo
                .find_by(search)
                .map_err(|e| e.context("Service Countries, find_by endpoint error occured.").into())
        })
    }

    /// Creates new country
    fn create_country(&self, new_country: NewCountry) -> ServiceFuture<Country> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let countries_repo = repo_factory.create_countries_repo(&*conn, user_id);
            conn.transaction::<(Country), FailureError, _>(move || countries_repo.create(new_country))
                .map_err(|e| e.context("Service Countries, create endpoint error occured.").into())
        })
    }

    /// Returns all countries
    fn get_all(&self) -> ServiceFuture<Country> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let countries_repo = repo_factory.create_countries_repo(&*conn, user_id);
            countries_repo
                .get_all()
                .map_err(|e| e.context("Service Countries, get_all endpoint error occured.").into())
        })
    }

    /// Returns all countries as a flat Vec
    fn get_all_flatten(&self) -> ServiceFuture<Vec<Country>> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let countries_repo = repo_factory.create_countries_repo(&*conn, user_id);
            countries_repo
                .get_all_flatten()
                .map_err(|e| e.context("Service Countries, get_all_flatten endpoint error occured.").into())
        })
    }
}
