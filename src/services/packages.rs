//! Packages Services, presents CRUD operations with countries

use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use r2d2::ManageConnection;

use failure::Error as FailureError;

use stq_types::{Alpha3, PackageId};

use super::types::{Service, ServiceFuture};
use models::packages::{NewPackages, Packages, UpdatePackages};
use repos::countries::get_all_parent_codes;
use repos::ReposFactory;

pub trait PackagesService {
    /// Create a new packages
    fn create_package(&self, payload: NewPackages) -> ServiceFuture<Packages>;

    /// Returns list of packages supported by the country
    fn find_packages_by_country(&self, country: Alpha3) -> ServiceFuture<Vec<Packages>>;

    /// Returns list of packages
    fn list_packages(&self) -> ServiceFuture<Vec<Packages>>;

    fn find_packages(&self, id_arg: PackageId) -> ServiceFuture<Option<Packages>>;

    /// Update a packages
    fn update_package(&self, id: PackageId, payload: UpdatePackages) -> ServiceFuture<Packages>;

    /// Delete a packages
    fn delete_package(&self, id: PackageId) -> ServiceFuture<Packages>;
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > PackagesService for Service<T, M, F>
{
    fn create_package(&self, payload: NewPackages) -> ServiceFuture<Packages> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let packages_repo = repo_factory.create_packages_repo(&*conn, user_id);
            conn.transaction::<Packages, FailureError, _>(move || {
                packages_repo
                    .create(payload)
                    .map_err(|e| e.context("Service Packages, create endpoint error occured.").into())
            })
        })
    }

    fn find_packages_by_country(&self, country: Alpha3) -> ServiceFuture<Vec<Packages>> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let packages_repo = repo_factory.create_packages_repo(&*conn, user_id);
            let countries_repo = repo_factory.create_countries_repo(&*conn, user_id);
            countries_repo
                .get_all()
                .and_then(|countries| {
                    let mut countries_list = vec![];
                    get_all_parent_codes(&countries, &country, &mut countries_list);
                    packages_repo.find_deliveries_to(countries_list)
                })
                .map_err(|e| e.context("Service Packages, find_deliveries_to endpoint error occured.").into())
        })
    }

    /// Returns list of packages
    fn list_packages(&self) -> ServiceFuture<Vec<Packages>> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let packages_repo = repo_factory.create_packages_repo(&*conn, user_id);
            packages_repo
                .list()
                .map_err(|e| e.context("Service Packages, list endpoint error occured.").into())
        })
    }

    fn find_packages(&self, id_arg: PackageId) -> ServiceFuture<Option<Packages>> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let packages_repo = repo_factory.create_packages_repo(&*conn, user_id);
            packages_repo
                .find(id_arg)
                .map_err(|e| e.context("Service Packages, find endpoint error occured.").into())
        })
    }

    fn update_package(&self, id: PackageId, payload: UpdatePackages) -> ServiceFuture<Packages> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let packages_repo = repo_factory.create_packages_repo(&*conn, user_id);
            packages_repo
                .update(id, payload)
                .map_err(|e| e.context("Service Packages, update endpoint error occured.").into())
        })
    }

    fn delete_package(&self, id: PackageId) -> ServiceFuture<Packages> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let packages_repo = repo_factory.create_packages_repo(&*conn, user_id);
            packages_repo
                .delete(id)
                .map_err(|e| e.context("Service Packages, delete endpoint error occured.").into())
        })
    }
}

#[cfg(test)]
pub mod tests {
    use std::sync::Arc;
    use tokio_core::reactor::Core;

    use stq_types::*;

    use models::*;
    use repos::repo_factory::tests::*;
    use services::packages::PackagesService;

    pub fn create_new_packages(name: String) -> NewPackages {
        NewPackages {
            name,
            max_size: 0,
            min_size: 0,
            max_weight: 0,
            min_weight: 0,
            deliveries_to: vec![],
        }
    }

    #[test]
    fn test_get_packages() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let work = service.find_packages(PackageId(1));
        let result = core.run(work).unwrap();
        assert_eq!(result.unwrap().id, PackageId(1));
    }

    #[test]
    fn test_create_packages() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let new_packages = create_new_packages("package1".to_string());
        let work = service.create_package(new_packages);
        let result = core.run(work).unwrap();
        assert_eq!(result.name, "package1".to_string());
    }

}
