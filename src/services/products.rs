//! Products Service, presents CRUD operations
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Error as FailureError;

use r2d2::ManageConnection;

use stq_types::{Alpha3, BaseProductId, CompanyPackageId, ShippingId};

use models::{
    AvailablePackageForUser, AvailableShippingForUser, NewProducts, NewShipping, Products, Shipping, ShippingProducts, UpdateProducts,
};
use repos::countries::create_tree_used_countries;
use repos::products::ProductsWithAvailableCountries;
use repos::ReposFactory;
use services::types::{Service, ServiceFuture};

pub trait ProductsService {
    /// Creates new products
    fn create_products(&self, payload: NewProducts) -> ServiceFuture<Products>;

    /// Delete and Insert shipping values
    fn upsert(&self, base_product_id: BaseProductId, payload: NewShipping) -> ServiceFuture<Shipping>;

    /// Get products
    fn get_by_base_product_id(&self, base_product_id: BaseProductId) -> ServiceFuture<Shipping>;

    /// find available product delivery to users country
    fn find_available_shipping_for_user(
        &self,
        base_product_id: BaseProductId,
        user_country: Alpha3,
    ) -> ServiceFuture<AvailableShippingForUser>;

    /// Update a product
    fn update_products(
        &self,
        base_product_id_arg: BaseProductId,
        company_package_id: CompanyPackageId,
        payload: UpdateProducts,
    ) -> ServiceFuture<Products>;

    /// Returns available package for user by id
    fn get_available_package_for_user(
        &self,
        base_product_id: BaseProductId,
        package_id: CompanyPackageId,
    ) -> ServiceFuture<Option<AvailablePackageForUser>>;

    /// Returns available package for user by shipping id
    fn get_available_package_for_user_by_shipping_id(&self, shipping_id: ShippingId) -> ServiceFuture<Option<AvailablePackageForUser>>;

    fn delete_products(&self, base_product_id_arg: BaseProductId) -> ServiceFuture<()>;
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > ProductsService for Service<T, M, F>
{
    fn create_products(&self, payload: NewProducts) -> ServiceFuture<Products> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let products_repo = repo_factory.create_products_repo(&*conn, user_id);
            conn.transaction::<Products, FailureError, _>(move || {
                products_repo
                    .create(payload)
                    .map_err(|e| e.context("Service Products, create endpoint error occured.").into())
            })
        })
    }

    fn upsert(&self, base_product_id: BaseProductId, payload: NewShipping) -> ServiceFuture<Shipping> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            conn.transaction::<Shipping, _, _>(|| {
                let products_repo = repo_factory.create_products_repo(&*conn, user_id);
                let pickups_repo = repo_factory.create_pickups_repo(&*conn, user_id);
                let countries_repo = repo_factory.create_countries_repo(&*conn, user_id);
                let pickup = payload.pickup.clone();
                products_repo
                    .delete(base_product_id)
                    .and_then(|_| products_repo.create_many(payload.items))
                    .and_then(|_| products_repo.get_products_countries(base_product_id))
                    .and_then(|products_with_countries| {
                        countries_repo.get_all().map(|countries| {
                            // getting all countries
                            products_with_countries
                                .into_iter()
                                .map(|product_with_countries| {
                                    // getting product with chosen package deliveries to
                                    let ProductsWithAvailableCountries(product, _) = product_with_countries;
                                    let deliveries_to = create_tree_used_countries(&countries, &product.deliveries_to);

                                    ShippingProducts { product, deliveries_to }
                                }).collect::<Vec<ShippingProducts>>()
                        })
                    }).and_then(|products| {
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
            }).map_err(|e: FailureError| e.context("Service Products, upsert endpoint error occured.").into())
        })
    }

    fn get_by_base_product_id(&self, base_product_id: BaseProductId) -> ServiceFuture<Shipping> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
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
                                let ProductsWithAvailableCountries(product, _) = product_with_countries;
                                // at first - take all package deliveries to country labels and make Vec of Country
                                let deliveries_to = create_tree_used_countries(&countries, &product.deliveries_to);
                                ShippingProducts { product, deliveries_to }
                            }).collect::<Vec<ShippingProducts>>()
                    })
                }).and_then(|products| {
                    pickups_repo.get(base_product_id).map(|pickups| Shipping {
                        items: products,
                        pickup: pickups,
                    })
                }).map_err(|e| {
                    e.context("Service Products, get_by_base_product_id endpoint error occurred.")
                        .into()
                })
        })
    }

    /// find available product delivery to users country
    fn find_available_shipping_for_user(
        &self,
        base_product_id: BaseProductId,
        user_country: Alpha3,
    ) -> ServiceFuture<AvailableShippingForUser> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let products_repo = repo_factory.create_products_repo(&*conn, user_id);
            let pickups_repo = repo_factory.create_pickups_repo(&*conn, user_id);
            products_repo
                .find_available_to(base_product_id, user_country)
                .and_then(|packages| {
                    pickups_repo
                        .get(base_product_id)
                        .map(|pickups| AvailableShippingForUser { packages, pickups })
                }).map_err(|e| e.context("Service Products, find_available_to endpoint error occurred.").into())
        })
    }

    /// Returns available package for user by id
    fn get_available_package_for_user(
        &self,
        base_product_id: BaseProductId,
        package_id: CompanyPackageId,
    ) -> ServiceFuture<Option<AvailablePackageForUser>> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let products_repo = repo_factory.create_products_repo(&*conn, user_id);

            products_repo
                .get_available_package_for_user(base_product_id, package_id)
                .map_err(|e| {
                    e.context("Service Products, get_available_package_for_user endpoint error occurred.")
                        .into()
                })
        })
    }

    /// Returns available package for user by shipping id
    fn get_available_package_for_user_by_shipping_id(&self, shipping_id: ShippingId) -> ServiceFuture<Option<AvailablePackageForUser>> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let products_repo = repo_factory.create_products_repo(&*conn, user_id);

            products_repo
                .get_available_package_for_user_by_shipping_id(shipping_id)
                .map_err(|e| {
                    e.context("Service Products, get_available_package_for_user_by_shipping_id endpoint error occurred.")
                        .into()
                })
        })
    }

    fn update_products(
        &self,
        base_product_id_arg: BaseProductId,
        company_package_id: CompanyPackageId,
        payload: UpdateProducts,
    ) -> ServiceFuture<Products> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let products_repo = repo_factory.create_products_repo(&*conn, user_id);
            products_repo
                .update(base_product_id_arg, company_package_id, payload)
                .map_err(|e| e.context("Service Products, update endpoint error occured.").into())
        })
    }

    fn delete_products(&self, base_product_id_arg: BaseProductId) -> ServiceFuture<()> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            conn.transaction::<(), _, _>(|| {
                let products_repo = repo_factory.create_products_repo(&*conn, user_id);
                let pickups_repo = repo_factory.create_pickups_repo(&*conn, user_id);
                products_repo
                    .delete(base_product_id_arg)
                    .and_then(|_| pickups_repo.delete(base_product_id_arg).and_then(|_| Ok(())))
            }).map_err(|e| e.context("Service Products, delete endpoint error occured.").into())
        })
    }
}
