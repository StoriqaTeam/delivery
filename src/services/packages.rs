//! Packages Services, presents CRUD operations with countries

use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Fail;
use futures::future::*;
use futures_cpupool::CpuPool;
use r2d2::{ManageConnection, Pool};

use stq_types::{CountryLabel, PackageId, UserId};

use errors::Error;

use super::types::ServiceFuture;
use models::packages::{NewPackages, Packages, UpdatePackages};
use repos::ReposFactory;

pub trait PackagesService {
    /// Create a new packages
    fn create(&self, payload: NewPackages) -> ServiceFuture<Packages>;

    /// Returns list of packages supported by the country
    fn find_deliveries_to(&self, country: CountryLabel) -> ServiceFuture<Vec<Packages>>;

    /// Returns list of packages
    fn list(&self) -> ServiceFuture<Vec<Packages>>;

    fn find(&self, id_arg: PackageId) -> ServiceFuture<Option<Packages>>;

    /// Update a packages
    fn update(&self, id: PackageId, payload: UpdatePackages) -> ServiceFuture<Packages>;

    /// Delete a packages
    fn delete(&self, id: PackageId) -> ServiceFuture<Packages>;
}

/// Packages services, responsible for Packages-related CRUD operations
pub struct PackagesServiceImpl<
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
    > PackagesServiceImpl<T, M, F>
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
    > PackagesService for PackagesServiceImpl<T, M, F>
{
    fn create(&self, payload: NewPackages) -> ServiceFuture<Packages> {
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
                            let packages_repo = repo_factory.create_packages_repo(&*conn, user_id);
                            packages_repo.create(payload)
                        })
                })
                .map_err(|e| e.context("Service Packages, create endpoint error occured.").into()),
        )
    }

    fn find_deliveries_to(&self, country: CountryLabel) -> ServiceFuture<Vec<Packages>> {
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
                            let packages_repo = repo_factory.create_packages_repo(&*conn, user_id);
                            packages_repo.find_deliveries_to(vec![country]) // TODO: take from countries tree all path
                        })
                })
                .map_err(|e| e.context("Service Packages, find_deliveries_to endpoint error occured.").into()),
        )
    }

    /// Returns list of packages
    fn list(&self) -> ServiceFuture<Vec<Packages>> {
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
                            let packages_repo = repo_factory.create_packages_repo(&*conn, user_id);
                            packages_repo.list()
                        })
                })
                .map_err(|e| e.context("Service Packages, list endpoint error occured.").into()),
        )
    }

    fn find(&self, id_arg: PackageId) -> ServiceFuture<Option<Packages>> {
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
                            let packages_repo = repo_factory.create_packages_repo(&*conn, user_id);
                            packages_repo.find(id_arg)
                        })
                })
                .map_err(|e| e.context("Service Packages, find endpoint error occured.").into()),
        )
    }

    fn update(&self, id: PackageId, payload: UpdatePackages) -> ServiceFuture<Packages> {
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
                            let packages_repo = repo_factory.create_packages_repo(&*conn, user_id);
                            packages_repo.update(id, payload)
                        })
                })
                .map_err(|e| e.context("Service Packages, update endpoint error occured.").into()),
        )
    }

    fn delete(&self, id: PackageId) -> ServiceFuture<Packages> {
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
                            let packages_repo = repo_factory.create_packages_repo(&*conn, user_id);
                            packages_repo.delete(id)
                        })
                })
                .map_err(|e| e.context("Service Packages, delete endpoint error occured.").into()),
        )
    }
}

#[cfg(test)]
pub mod tests {
    use futures_cpupool::CpuPool;
    use r2d2;
    use tokio_core::reactor::Core;

    use stq_types::*;

    use super::*;
    use models::*;
    use repos::repo_factory::tests::*;

    fn create_packages_service(user_id: Option<UserId>) -> PackagesServiceImpl<MockConnection, MockConnectionManager, ReposFactoryMock> {
        let manager = MockConnectionManager::default();
        let db_pool = r2d2::Pool::builder().build(manager).expect("Failed to create connection pool");
        let cpu_pool = CpuPool::new(1);

        PackagesServiceImpl {
            db_pool: db_pool,
            cpu_pool: cpu_pool,
            user_id: user_id,
            repo_factory: MOCK_REPO_FACTORY,
        }
    }

    pub fn create_new_packages(name: String) -> NewPackages {
        NewPackages {
            name,
            max_size: 0f64,
            min_size: 0f64,
            max_weight: 0f64,
            min_weight: 0f64,
            deliveries_to: vec![],
        }
    }

    #[test]
    fn test_get_packages() {
        let mut core = Core::new().unwrap();
        let service = create_packages_service(Some(MOCK_USER_ID));
        let work = service.find(PackageId(1));
        let result = core.run(work).unwrap();
        assert_eq!(result.unwrap().id, PackageId(1));
    }

    #[test]
    fn test_create_packages() {
        let mut core = Core::new().unwrap();
        let service = create_packages_service(Some(MOCK_USER_ID));
        let new_packages = create_new_packages("package1".to_string());
        let work = service.create(new_packages);
        let result = core.run(work).unwrap();
        assert_eq!(result.name, "package1".to_string());
    }

}
