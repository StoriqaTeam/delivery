//! Products Service, presents CRUD operations
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Error as FailureError;
use validator::Validate;

use r2d2::ManageConnection;

use stq_types::{Alpha3, BaseProductId, CompanyPackageId, ProductPrice, ShippingId};

use errors::Error;
use models::{
    AvailablePackageForUser, AvailableShippingForUser, NewProductValidation, NewProducts, NewShipping, PackageValidation, Products,
    ShipmentMeasurements, Shipping, ShippingProducts, ShippingRateSource, ShippingValidation, UpdateProducts,
};
use repos::companies_packages::CompaniesPackagesRepo;
use repos::countries::create_tree_used_countries;
use repos::products::ProductsWithAvailableCountries;
use repos::shipping_rates::ShippingRatesRepo;
use repos::ReposFactory;
use services::types::{Service, ServiceFuture};

pub trait ProductsService {
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

    /// find available product delivery to user's country with correct prices
    fn find_available_shipping_for_user_v2(
        &self,
        base_product_id: BaseProductId,
        delivery_from: Alpha3,
        delivery_to: Alpha3,
        volume: u32,
        weight: u32,
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

    /// Returns available package for user by id with correct price
    fn get_available_package_for_user_v2(
        &self,
        base_product_id: BaseProductId,
        package_id: CompanyPackageId,
        delivery_from: Alpha3,
        delivery_to: Alpha3,
        volume: u32,
        weight: u32,
    ) -> ServiceFuture<Option<AvailablePackageForUser>>;

    /// Returns available package for user by shipping id
    fn get_available_package_for_user_by_shipping_id(&self, shipping_id: ShippingId) -> ServiceFuture<Option<AvailablePackageForUser>>;

    /// Returns available package for user by shipping id with correct price
    fn get_available_package_for_user_by_shipping_id_v2(
        &self,
        shipping_id: ShippingId,
        delivery_from: Alpha3,
        delivery_to: Alpha3,
        volume: u32,
        weight: u32,
    ) -> ServiceFuture<Option<AvailablePackageForUser>>;

    fn delete_products(&self, base_product_id_arg: BaseProductId) -> ServiceFuture<()>;
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > ProductsService for Service<T, M, F>
{
    fn upsert(&self, base_product_id: BaseProductId, payload: NewShipping) -> ServiceFuture<Shipping> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            conn.transaction::<Shipping, _, _>(|| {
                let products_repo = repo_factory.create_products_repo(&*conn, user_id);
                let pickups_repo = repo_factory.create_pickups_repo(&*conn, user_id);
                let countries_repo = repo_factory.create_countries_repo(&*conn, user_id);
                let companies_repo = repo_factory.create_companies_repo(&*conn, user_id);
                let packages_repo = repo_factory.create_packages_repo(&*conn, user_id);
                let company_packages_repo = repo_factory.create_companies_packages_repo(&*conn, user_id);
                let pickup = payload.pickup.clone();

                products_repo
                    .delete(base_product_id)
                    .and_then(|_| {
                        payload
                            .items
                            .clone()
                            .into_iter()
                            .map(|new_product| {
                                let company_package = company_packages_repo.get(new_product.company_package_id)?.ok_or(Error::Validate(
                                    validation_errors!({
                                        "company_package_id": ["company_package_id" => "Company package not found"]
                                    }),
                                ))?;
                                let company = companies_repo
                                    .find(company_package.company_id)?
                                    .ok_or(format_err!("Company with id = {} not found", company_package.company_id))?;
                                let package = packages_repo
                                    .find(company_package.package_id)?
                                    .ok_or(format_err!("Package with id = {} not found", company_package.package_id))?;

                                let package_validation = new_product.measurements.clone().map(|measurements| PackageValidation {
                                    measurements,
                                    package: package.clone(),
                                });

                                NewProductValidation {
                                    product: new_product.clone(),
                                    package: package_validation,
                                    shipping: ShippingValidation {
                                        delivery_from: new_product.delivery_from.clone(),
                                        deliveries_to: new_product.deliveries_to.clone(),
                                        company,
                                        package,
                                    },
                                }.validate()
                                .map(|_| new_product)
                                .map_err(|e| FailureError::from(Error::Validate(e)))
                            }).collect::<Result<Vec<NewProducts>, _>>()?;

                        products_repo.create_many(payload.items)
                    }).and_then(|_| products_repo.get_products_countries(base_product_id))
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

    /// find available product delivery to user's country with correct prices
    fn find_available_shipping_for_user_v2(
        &self,
        base_product_id: BaseProductId,
        delivery_from: Alpha3,
        delivery_to: Alpha3,
        volume: u32,
        weight: u32,
    ) -> ServiceFuture<AvailableShippingForUser> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let products_repo = repo_factory.create_products_repo(&*conn, user_id);
            let company_package_repo = repo_factory.create_companies_packages_repo(&*conn, user_id);
            let shipping_rates_repo = repo_factory.create_shipping_rates_repo(&*conn, user_id);
            let pickups_repo = repo_factory.create_pickups_repo(&*conn, user_id);

            let run = || {
                let packages = products_repo
                    .find_available_to(base_product_id, delivery_to.clone())?
                    .into_iter()
                    .map(|pkg| {
                        with_price_from_rates(
                            &*company_package_repo,
                            &*shipping_rates_repo,
                            delivery_from.clone(),
                            delivery_to.clone(),
                            volume,
                            weight,
                            pkg,
                        )
                    }).collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    .filter_map(|x| x)
                    .collect::<Vec<_>>();

                pickups_repo
                    .get(base_product_id)
                    .map(|pickups| AvailableShippingForUser { packages, pickups })
            };

            run().map_err(|e: FailureError| e.context("Service Products, find_available_to endpoint error occurred.").into())
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
                .get_available_package_for_user(base_product_id, package_id, None)
                .map_err(|e| {
                    e.context("Service Products, get_available_package_for_user endpoint error occurred.")
                        .into()
                })
        })
    }

    /// Returns available package for user by id with correct price
    fn get_available_package_for_user_v2(
        &self,
        base_product_id: BaseProductId,
        company_package_id: CompanyPackageId,
        delivery_from: Alpha3,
        delivery_to: Alpha3,
        volume: u32,
        weight: u32,
    ) -> ServiceFuture<Option<AvailablePackageForUser>> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let products_repo = repo_factory.create_products_repo(&*conn, user_id);
            let company_package_repo = repo_factory.create_companies_packages_repo(&*conn, user_id);
            let shipping_rates_repo = repo_factory.create_shipping_rates_repo(&*conn, user_id);

            let run = || {
                let pkg_for_user =
                    products_repo.get_available_package_for_user(base_product_id, company_package_id, Some(delivery_to.clone()))?;
                let pkg_for_user = match pkg_for_user {
                    None => {
                        return Ok(None);
                    }
                    Some(pkg) => pkg,
                };
                with_price_from_rates(
                    &*company_package_repo,
                    &*shipping_rates_repo,
                    delivery_from,
                    delivery_to,
                    volume,
                    weight,
                    pkg_for_user,
                )
            };

            run().map_err(|e: FailureError| {
                e.context("Service Products, get_available_package_for_user_v2 endpoint error occurred.")
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
                .get_available_package_for_user_by_shipping_id(shipping_id, None)
                .map_err(|e| {
                    e.context("Service Products, get_available_package_for_user_by_shipping_id endpoint error occurred.")
                        .into()
                })
        })
    }

    /// Returns available package for user by shipping id with correct price
    fn get_available_package_for_user_by_shipping_id_v2(
        &self,
        shipping_id: ShippingId,
        delivery_from: Alpha3,
        delivery_to: Alpha3,
        volume: u32,
        weight: u32,
    ) -> ServiceFuture<Option<AvailablePackageForUser>> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let products_repo = repo_factory.create_products_repo(&*conn, user_id);
            let company_package_repo = repo_factory.create_companies_packages_repo(&*conn, user_id);
            let shipping_rates_repo = repo_factory.create_shipping_rates_repo(&*conn, user_id);

            let run = || {
                let pkg_for_user = products_repo.get_available_package_for_user_by_shipping_id(shipping_id, Some(delivery_to.clone()))?;
                let pkg_for_user = match pkg_for_user {
                    None => {
                        return Ok(None);
                    }
                    Some(pkg) => pkg,
                };
                with_price_from_rates(
                    &*company_package_repo,
                    &*shipping_rates_repo,
                    delivery_from,
                    delivery_to,
                    volume,
                    weight,
                    pkg_for_user,
                )
            };

            run().map_err(|e: FailureError| {
                e.context("Service Products, get_available_package_for_user_by_shipping_id_v2 endpoint error occurred.")
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

fn with_price_from_rates<'a>(
    company_package_repo: &'a CompaniesPackagesRepo,
    shipping_rates_repo: &'a ShippingRatesRepo,
    delivery_from: Alpha3,
    delivery_to: Alpha3,
    volume: u32,
    weight: u32,
    mut pkg_for_user: AvailablePackageForUser,
) -> Result<Option<AvailablePackageForUser>, FailureError> {
    if pkg_for_user.price.is_some() {
        return Ok(Some(pkg_for_user));
    }

    let company_package_id = pkg_for_user.id;
    let company_package = company_package_repo
        .get(company_package_id)?
        .ok_or(format_err!("Company package with id {} not found", company_package_id))?;

    let price = match company_package.shipping_rate_source {
        ShippingRateSource::NotAvailable => None,
        ShippingRateSource::Static { dimensional_factor } => shipping_rates_repo
            .get_rates(company_package_id, delivery_from, delivery_to)?
            .and_then(|rates| {
                let measurements = ShipmentMeasurements {
                    volume_cubic_cm: volume,
                    weight_g: weight,
                };
                rates.calculate_delivery_price(measurements, dimensional_factor).map(ProductPrice)
            }),
    };

    Ok(price.map(|price| {
        pkg_for_user.price = Some(price);
        pkg_for_user
    }))
}
