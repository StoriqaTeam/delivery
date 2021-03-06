//! Repo Packages table.

use diesel;
use diesel::connection::AnsiTransactionManager;
use diesel::dsl::sql;
use diesel::pg::types::sql_types::Array;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::LoadQuery;
use diesel::query_dsl::RunQueryDsl;
use diesel::sql_types::VarChar;
use diesel::Connection;

use errors::Error;
use failure::Error as FailureError;

use stq_types::{Alpha3, PackageId, UserId};

use models::authorization::*;
use models::countries::Country;
use models::packages::{NewPackages, Packages, PackagesRaw, UpdatePackages};
use repos::legacy_acl::*;
use repos::types::RepoResult;
use repos::*;

use schema::packages::dsl::*;

/// Packages repository for handling Packages
pub trait PackagesRepo {
    /// Create a new packages
    fn create(&self, payload: NewPackages) -> RepoResult<Packages>;

    /// Returns list of packages supported by the country
    fn find_deliveries_to(&self, countries: Vec<Alpha3>) -> RepoResult<Vec<Packages>>;

    /// Returns list of packages
    fn list(&self) -> RepoResult<Vec<Packages>>;

    /// Find specific package by ID
    fn find(&self, id_arg: PackageId) -> RepoResult<Option<Packages>>;

    /// Update a packages
    fn update(&self, id: PackageId, payload: UpdatePackages) -> RepoResult<Packages>;

    /// Delete a packages
    fn delete(&self, id: PackageId) -> RepoResult<Packages>;
}

/// Implementation of UserRoles trait
pub struct PackagesRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<Acl<Resource, Action, Scope, FailureError, Packages>>,
    pub countries: Country,
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> PackagesRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<Acl<Resource, Action, Scope, FailureError, Packages>>, countries: Country) -> Self {
        Self { db_conn, acl, countries }
    }

    fn execute_query<Ty: Send + 'static, U: LoadQuery<T, Ty> + Send + 'static>(&self, query: U) -> RepoResult<Ty> {
        query.get_result::<Ty>(self.db_conn).map_err(|e| Error::from(e).into())
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> PackagesRepo for PackagesRepoImpl<'a, T> {
    fn create(&self, payload: NewPackages) -> RepoResult<Packages> {
        debug!("create new packages_ {:?}.", payload);
        let payload = payload.to_raw()?;

        let query = diesel::insert_into(packages).values(&payload);
        query
            .get_result::<PackagesRaw>(self.db_conn)
            .map_err(|e| Error::from(e).into())
            .and_then(|p| p.to_packages(&self.countries))
            .and_then(|packages_| {
                acl::check(&*self.acl, Resource::Packages, Action::Create, self, Some(&packages_)).and_then(|_| Ok(packages_))
            })
            .map_err(|e: FailureError| e.context(format!("create new packages_ {:?}.", payload)).into())
    }

    /// Returns list of packages supported by the country
    fn find_deliveries_to(&self, countries: Vec<Alpha3>) -> RepoResult<Vec<Packages>> {
        debug!("Find in packages with country {:?}.", countries);

        let pg_countries: Vec<String> = countries.iter().cloned().map(|c| c.0).collect();

        let query = packages.filter(sql("deliveries_to ?| ").bind::<Array<VarChar>, _>(pg_countries));

        query
            .get_results(self.db_conn)
            .map_err(|e| Error::from(e).into())
            .and_then(|packages_raw: Vec<PackagesRaw>| {
                packages_raw
                    .into_iter()
                    .map(|packages_raw| packages_raw.to_packages(&self.countries))
                    .collect()
            })
            .and_then(|packages_res: Vec<Packages>| {
                for packages_ in &packages_res {
                    acl::check(&*self.acl, Resource::Packages, Action::Read, self, Some(&packages_))?;
                }
                Ok(packages_res)
            })
            .map_err(|e: FailureError| {
                e.context(format!("Find in packages with country {:?} error occured", countries))
                    .into()
            })
    }

    /// Returns list of packages
    fn list(&self) -> RepoResult<Vec<Packages>> {
        debug!("List packages");

        let query = packages.order(id);

        query
            .get_results(self.db_conn)
            .map_err(|e| Error::from(e).into())
            .and_then(|raws: Vec<PackagesRaw>| raws.into_iter().map(|raw| raw.to_packages(&self.countries)).collect())
            .and_then(|results: Vec<Packages>| {
                for package in &results {
                    acl::check(&*self.acl, Resource::Packages, Action::Read, self, Some(&package))?;
                }
                Ok(results)
            })
            .map_err(|e: FailureError| e.context("Find in packages error occured").into())
    }

    /// Find specific package by ID
    fn find(&self, id_arg: PackageId) -> RepoResult<Option<Packages>> {
        debug!("Find in package with id {}.", id_arg);

        let query = packages.find(id_arg);
        query
            .get_result::<PackagesRaw>(self.db_conn)
            .optional()
            .map_err(|e| Error::from(e).into())
            .and_then(|raw: Option<PackagesRaw>| match raw {
                Some(value) => {
                    let package = value.to_packages(&self.countries)?;
                    acl::check(&*self.acl, Resource::Packages, Action::Read, self, Some(&package))?;
                    Ok(Some(package))
                }
                None => Ok(None),
            })
            .map_err(|e: FailureError| e.context(format!("Find package with id: {} error occured", id_arg)).into())
    }

    fn update(&self, id_arg: PackageId, payload: UpdatePackages) -> RepoResult<Packages> {
        debug!("Updating packages_ payload {:?}.", payload);
        let payload = payload.to_raw()?;

        self.execute_query(packages.filter(id.eq(id_arg)))
            .and_then(|packages_: PackagesRaw| packages_.to_packages(&self.countries))
            .and_then(|packages_: Packages| acl::check(&*self.acl, Resource::Packages, Action::Update, self, Some(&packages_)))
            .and_then(|_| {
                let filtered = packages.filter(id.eq(id_arg));

                let query = diesel::update(filtered).set(payload.clone());
                query
                    .get_result::<PackagesRaw>(self.db_conn)
                    .map_err(|e| Error::from(e).into())
                    .and_then(|packages_: PackagesRaw| packages_.to_packages(&self.countries))
            })
            .map_err(|e: FailureError| e.context(format!("Updating packages payload {:?} failed.", payload)).into())
    }

    fn delete(&self, id_arg: PackageId) -> RepoResult<Packages> {
        debug!("delete packages_ id: {}.", id_arg);

        acl::check(&*self.acl, Resource::Packages, Action::Delete, self, None)?;

        let filtered = packages.filter(id.eq(id_arg));
        let query = diesel::delete(filtered);
        query
            .get_result::<PackagesRaw>(self.db_conn)
            .map_err(|e| Error::from(e).into())
            .and_then(|packages_: PackagesRaw| packages_.to_packages(&self.countries))
            .map_err(move |e| e.context(format!("delete packages id: {}.", id_arg)).into())
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, Packages>
    for PackagesRepoImpl<'a, T>
{
    fn is_in_scope(&self, _user_id: UserId, scope: &Scope, _obj: Option<&Packages>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => false,
        }
    }
}
