//! Products Service, presents CRUD operations
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Error as FailureError;
use failure::Fail;
use futures::future::*;
use futures_cpupool::CpuPool;
use r2d2::{ManageConnection, Pool};

use stq_types::{Alpha3, BaseProductId, CompanyPackageId, UserId};

use errors::Error;
use models::{AvailableShipppingForUser, Country, NewProducts, NewShipping, Products, Shipping, ShippingProducts, UpdateProducts};
use repos::countries::{get_country, set_selected};
use repos::products::ProductsWithAvailableCountries;
use repos::ReposFactory;
use services::types::ServiceFuture;

pub trait ProductsService {
    /// Creates new products
    fn create(&self, payload: NewProducts) -> ServiceFuture<Products>;

    /// Delete and Insert shipping values
    fn upsert(&self, base_product_id: BaseProductId, payload: NewShipping) -> ServiceFuture<Shipping>;

    /// Get products
    fn get_by_base_product_id(&self, base_product_id: BaseProductId) -> ServiceFuture<Shipping>;

    /// find available product delivery to users country
    fn find_available_to(&self, base_product_id: BaseProductId, user_country: Alpha3) -> ServiceFuture<AvailableShipppingForUser>;

    /// Update a product
    fn update(
        &self,
        base_product_id_arg: BaseProductId,
        company_package_id: CompanyPackageId,
        payload: UpdateProducts,
    ) -> ServiceFuture<Products>;

    fn delete(&self, base_product_id_arg: BaseProductId) -> ServiceFuture<()>;
}

/// Products services, responsible for CRUD operations
pub struct ProductsServiceImpl<
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
    > ProductsServiceImpl<T, M, F>
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
    > ProductsService for ProductsServiceImpl<T, M, F>
{
    fn create(&self, payload: NewProducts) -> ServiceFuture<Products> {
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
                            let products_repo = repo_factory.create_products_repo(&*conn, user_id);
                            products_repo.create(payload)
                        })
                })
                .map_err(|e| e.context("Service Products, create endpoint error occured.").into()),
        )
    }

    fn upsert(&self, base_product_id: BaseProductId, payload: NewShipping) -> ServiceFuture<Shipping> {
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
                            conn.transaction::<Shipping, _, _>(|| {
                                let products_repo = repo_factory.create_products_repo(&*conn, user_id);
                                let pickups_repo = repo_factory.create_pickups_repo(&*conn, user_id);
                                let countries_repo = repo_factory.create_countries_repo(&*conn, user_id);
                                let pickup = payload.pickup.clone();
                                products_repo
                                    .delete(base_product_id.clone())
                                    .and_then(|_| products_repo.create_many(payload.items))
                                    .and_then(|_| products_repo.get_products_countries(base_product_id.clone()))
                                    .and_then(|products_with_countries| {
                                        countries_repo.get_all().map(|countries| {
                                            // getting all countries
                                            products_with_countries
                                                .into_iter()
                                                .map(|product_with_countries| {
                                                    // getting product with chosen package deliveries to
                                                    let ProductsWithAvailableCountries(product, package_countries) = product_with_countries;
                                                    // at first - take all package deliveries to country labels and make Vec of Country
                                                    let deliveries_to = package_countries
                                                        .into_iter()
                                                        .filter_map(|label| {
                                                            get_country(&countries, &label).map(|mut country| {
                                                                // now select only countries that in products deliveries to
                                                                set_selected(&mut country, &product.deliveries_to);
                                                                country
                                                            })
                                                        })
                                                        .collect::<Vec<Country>>();
                                                    ShippingProducts { product, deliveries_to }
                                                })
                                                .collect::<Vec<ShippingProducts>>()
                                        })
                                    })
                                    .and_then(|products| {
                                        if let Some(pickup) = pickup {
                                            pickups_repo
                                                .delete(base_product_id)
                                                .and_then(|_| pickups_repo.create(pickup))
                                                .map(Some)
                                        } else {
                                            Ok(None)
                                        }.map(|pickups| Shipping {
                                            items: products,
                                            pickup: pickups,
                                        })
                                    })
                            })
                        })
                })
                .map_err(|e: FailureError| e.context("Service Products, upsert endpoint error occured.").into()),
        )
    }

    fn get_by_base_product_id(&self, base_product_id: BaseProductId) -> ServiceFuture<Shipping> {
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
                            let products_repo = repo_factory.create_products_repo(&*conn, user_id);
                            let pickups_repo = repo_factory.create_pickups_repo(&*conn, user_id);
                            let countries_repo = repo_factory.create_countries_repo(&*conn, user_id);
                            products_repo
                                .get_products_countries(base_product_id)
                                .and_then(|products_with_countries| {
                                    countries_repo.get_all().map(|countries| {
                                        // getting all countries
                                        products_with_countries
                                            .into_iter()
                                            .map(|product_with_countries| {
                                                // getting product with chosen package deliveries to
                                                let ProductsWithAvailableCountries(product, package_countries) = product_with_countries;
                                                // at first - take all package deliveries to country labels and make Vec of Country
                                                let deliveries_to = package_countries
                                                    .into_iter()
                                                    .filter_map(|label| {
                                                        get_country(&countries, &label).map(|mut country| {
                                                            // now select only countries that in products deliveries to
                                                            set_selected(&mut country, &product.deliveries_to);
                                                            country
                                                        })
                                                    })
                                                    .collect::<Vec<Country>>();
                                                ShippingProducts { product, deliveries_to }
                                            })
                                            .collect::<Vec<ShippingProducts>>()
                                    })
                                })
                                .and_then(|products| {
                                    pickups_repo.get(base_product_id).map(|pickups| Shipping {
                                        items: products,
                                        pickup: pickups,
                                    })
                                })
                        })
                })
                .map_err(|e| e.context("Service Products, get_by_base_product_id endpoint error occured.").into()),
        )
    }

    /// find available product delivery to users country
    fn find_available_to(&self, base_product_id: BaseProductId, user_country: Alpha3) -> ServiceFuture<AvailableShipppingForUser> {
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
                            let products_repo = repo_factory.create_products_repo(&*conn, user_id);
                            let pickups_repo = repo_factory.create_pickups_repo(&*conn, user_id);
                            products_repo.find_available_to(base_product_id, user_country).and_then(|packages| {
                                pickups_repo
                                    .get(base_product_id)
                                    .map(|pickups| AvailableShipppingForUser { packages, pickups })
                            })
                        })
                })
                .map_err(|e| e.context("Service Products, find_available_to endpoint error occured.").into()),
        )
    }

    fn update(
        &self,
        base_product_id_arg: BaseProductId,
        company_package_id: CompanyPackageId,
        payload: UpdateProducts,
    ) -> ServiceFuture<Products> {
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
                            let products_repo = repo_factory.create_products_repo(&*conn, user_id);
                            products_repo.update(base_product_id_arg, company_package_id, payload)
                        })
                })
                .map_err(|e| e.context("Service Products, update endpoint error occured.").into()),
        )
    }

    fn delete(&self, base_product_id_arg: BaseProductId) -> ServiceFuture<()> {
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
                            conn.transaction::<(), _, _>(|| {
                                let products_repo = repo_factory.create_products_repo(&*conn, user_id);
                                let pickups_repo = repo_factory.create_pickups_repo(&*conn, user_id);
                                products_repo
                                    .delete(base_product_id_arg.clone())
                                    .and_then(|_| pickups_repo.delete(base_product_id_arg).and_then(|_| Ok(())))
                            })
                        })
                })
                .map_err(|e| e.context("Service Products, delete endpoint error occured.").into()),
        )
    }
}
