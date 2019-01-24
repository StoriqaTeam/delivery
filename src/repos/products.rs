//! REPO Products table. Products is an entity that
//! contains info about international shipping of base_product.

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
use failure::Error as FailureError;

use stq_types::{BaseProductId, CompanyPackageId, ShippingId, UserId};

use models::authorization::*;
use models::countries::Country;
use models::{
    AvailablePackageForUser, CompaniesPackagesRaw, CompanyRaw, NewProducts, NewProductsRaw, PackagesRaw, Products, ProductsRaw,
    ShippingVariant, UpdateProducts, UserRole,
};

use repos::legacy_acl::*;
use repos::types::RepoResult;
use repos::*;
use schema::companies::dsl as DslCompanies;
use schema::companies_packages::dsl as DslCompaniesPackages;
use schema::packages::dsl as DslPackages;
use schema::products::dsl as DslProducts;
use schema::roles::dsl as Roles;

pub struct ProductsWithAvailableCountries(pub Products, pub Vec<Alpha3>);

/// Products repository for handling Products
pub trait ProductsRepo {
    /// Create a new products
    fn create(&self, payload: NewProducts) -> RepoResult<Products>;

    /// Create a new products
    fn create_many(&self, payload: Vec<NewProducts>) -> RepoResult<Vec<Products>>;

    /// Get a products
    fn get_by_base_product_id(&self, base_product_id: BaseProductId) -> RepoResult<Vec<Products>>;

    /// Get a products with available countries for delivery by package
    fn get_products_countries(&self, base_product_id: BaseProductId) -> RepoResult<Vec<ProductsWithAvailableCountries>>;

    /// find available product delivery to users country
    fn find_available_to(&self, base_product_id: BaseProductId, user_country: Alpha3) -> RepoResult<Vec<AvailablePackageForUser>>;

    /// Update a products
    fn update(
        &self,
        base_product_id_arg: BaseProductId,
        company_package_id: CompanyPackageId,
        payload: UpdateProducts,
    ) -> RepoResult<Products>;

    /// Returns available package for user by id
    /// DEPRECATED. Use `get_available_package_for_user_by_shipping_id` instead.
    fn get_available_package_for_user(
        &self,
        base_product_id_arg: BaseProductId,
        package_id_arg: CompanyPackageId,
    ) -> RepoResult<Option<AvailablePackageForUser>>;

    /// Returns available package for user by shipping id
    fn get_available_package_for_user_by_shipping_id(
        &self,
        shipping_id_arg: ShippingId,
        delivery_to: Option<Alpha3>,
    ) -> RepoResult<Option<AvailablePackageForUser>>;

    /// Delete a products
    fn delete(&self, base_product_id_arg: BaseProductId) -> RepoResult<Vec<Products>>;
}

pub struct ProductsRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<Acl<Resource, Action, Scope, FailureError, Products>>,
    pub countries: Country,
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> ProductsRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<Acl<Resource, Action, Scope, FailureError, Products>>, countries: Country) -> Self {
        Self { db_conn, acl, countries }
    }

    fn execute_query<Ty: Send + 'static, U: LoadQuery<T, Ty> + Send + 'static>(&self, query: U) -> RepoResult<Ty> {
        query.get_result::<Ty>(self.db_conn).map_err(From::from)
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> ProductsRepo for ProductsRepoImpl<'a, T> {
    fn create(&self, payload: NewProducts) -> RepoResult<Products> {
        debug!("create new products {:?}.", payload);
        let payload = payload.to_raw()?;
        let query = diesel::insert_into(DslProducts::products).values(&payload);
        query
            .get_result::<ProductsRaw>(self.db_conn)
            .map_err(From::from)
            .and_then(|products_| products_.to_products())
            .and_then(|product| {
                acl::check(&*self.acl, Resource::Products, Action::Create, self, Some(&product))?;
                Ok(product)
            })
            .map_err(|e: FailureError| e.context(format!("create new products {:?}.", payload)).into())
    }

    fn create_many(&self, payload: Vec<NewProducts>) -> RepoResult<Vec<Products>> {
        debug!("create many new products {:?}.", payload);
        let payload = payload
            .into_iter()
            .map(|v| v.to_raw().map_err(From::from))
            .collect::<RepoResult<Vec<NewProductsRaw>>>()?;

        let query = diesel::insert_into(DslProducts::products).values(&payload);
        query
            .get_results::<ProductsRaw>(self.db_conn)
            .map_err(From::from)
            .and_then(|products_: Vec<ProductsRaw>| {
                let mut new_products = vec![];
                for product in products_ {
                    let product = product.to_products()?;
                    acl::check(&*self.acl, Resource::Products, Action::Create, self, Some(&product))?;
                    new_products.push(product);
                }

                new_products.sort_by(|a, b| a.id.cmp(&b.id));

                Ok(new_products)
            })
            .map_err(|e: FailureError| e.context(format!("create many new products {:?}.", payload)).into())
    }

    fn get_by_base_product_id(&self, base_product_id_arg: BaseProductId) -> RepoResult<Vec<Products>> {
        debug!("get products by base_product_id {:?}.", base_product_id_arg);
        let query = DslProducts::products
            .filter(DslProducts::base_product_id.eq(base_product_id_arg))
            .order(DslProducts::id);

        query
            .get_results(self.db_conn)
            .map_err(From::from)
            .and_then(|products_: Vec<ProductsRaw>| {
                let mut new_products = vec![];
                for product in products_ {
                    let product = product.to_products()?;
                    acl::check(&*self.acl, Resource::Products, Action::Read, self, Some(&product))?;
                    new_products.push(product);
                }
                Ok(new_products)
            })
            .map_err(|e: FailureError| {
                e.context(format!("Getting products with base_product_id {:?} failed.", base_product_id_arg))
                    .into()
            })
    }

    /// Get a products with countries from packages
    fn get_products_countries(&self, base_product_id_arg: BaseProductId) -> RepoResult<Vec<ProductsWithAvailableCountries>> {
        debug!(
            "Find in available countries for delivery by base_product_id: {:?}.",
            base_product_id_arg
        );

        let query = DslProducts::products
            .filter(DslProducts::base_product_id.eq(base_product_id_arg))
            .inner_join(DslCompaniesPackages::companies_packages.inner_join(DslPackages::packages))
            .order(DslPackages::id);

        query
            .get_results::<(ProductsRaw, (CompaniesPackagesRaw, PackagesRaw))>(self.db_conn)
            .map_err(From::from)
            .and_then(|results| {
                let mut data = vec![];
                for result in results {
                    let (product_raw, (_, package_raw)) = result;
                    let countries_codes = package_raw
                        .to_packages(&self.countries)?
                        .deliveries_to
                        .into_iter()
                        .map(|c| c.alpha3)
                        .collect();
                    let element = ProductsWithAvailableCountries(product_raw.to_products()?, countries_codes);

                    data.push(element);
                }
                Ok(data)
            })
            .map_err(|e: FailureError| {
                e.context(format!(
                    "Find in available countries for delivery by base_product_id: {:?} error occured",
                    base_product_id_arg
                ))
                .into()
            })
    }

    /// find available product delivery to users country
    fn find_available_to(&self, base_product_id_arg: BaseProductId, user_country: Alpha3) -> RepoResult<Vec<AvailablePackageForUser>> {
        debug!(
            "Find available product {} delivery to users country {}.",
            base_product_id_arg, user_country
        );

        let pg_countries: Vec<String> = vec![user_country.clone()].into_iter().map(|c| c.0).collect();

        let query = DslProducts::products
            .filter(DslProducts::base_product_id.eq(base_product_id_arg))
            .filter(sql("products.deliveries_to ?| ").bind::<Array<VarChar>, _>(pg_countries))
            .inner_join(
                DslCompaniesPackages::companies_packages
                    .inner_join(DslCompanies::companies)
                    .inner_join(DslPackages::packages),
            )
            .order(DslCompanies::label);

        query
            .get_results::<(ProductsRaw, (CompaniesPackagesRaw, CompanyRaw, PackagesRaw))>(self.db_conn)
            .map(|results| {
                let available_packages = results
                    .into_iter()
                    .map(|result| {
                        let (product_raw, (companies_package, company_raw, package_raw)) = result;
                        AvailablePackageForUser {
                            id: companies_package.id,
                            shipping_id: product_raw.id,
                            name: get_company_package_name(&company_raw.label, &package_raw.name),
                            logo: company_raw.logo.clone(),
                            price: product_raw.price,
                            currency: company_raw.currency,
                            shipping_variant: product_raw.shipping.clone(),
                            store_id: product_raw.store_id,
                            base_product_id: product_raw.base_product_id,
                        }
                    })
                    .collect::<Vec<_>>();

                let local_package_ids = available_packages
                    .iter()
                    .filter_map(|package| {
                        if package.shipping_variant.clone() == ShippingVariant::Local {
                            Some(package.id)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                available_packages
                    .into_iter()
                    .filter(|package| {
                        package.shipping_variant.clone() == ShippingVariant::Local || !local_package_ids.contains(&package.id)
                    })
                    .collect::<Vec<_>>()
            })
            .map_err(move |e| {
                FailureError::from(e)
                    .context(format!(
                        "Find available product {} delivery to users country {} failure.",
                        base_product_id_arg, user_country
                    ))
                    .into()
            })
    }

    /// Returns available package for user by id
    /// DEPRECATED. Use `get_available_package_for_user_by_shipping_id` instead.
    fn get_available_package_for_user(
        &self,
        base_product_id_arg: BaseProductId,
        package_id_arg: CompanyPackageId,
    ) -> RepoResult<Option<AvailablePackageForUser>> {
        debug!(
            "Get available package for base product: {} with select company package id: {}.",
            base_product_id_arg, package_id_arg
        );

        let query = DslProducts::products
            .inner_join(
                DslCompaniesPackages::companies_packages
                    .inner_join(DslCompanies::companies)
                    .inner_join(DslPackages::packages),
            )
            .filter(DslProducts::base_product_id.eq(base_product_id_arg))
            .filter(DslProducts::company_package_id.eq(package_id_arg))
            .order(DslCompanies::label);

        query
            .get_result::<(ProductsRaw, (CompaniesPackagesRaw, CompanyRaw, PackagesRaw))>(self.db_conn)
            .optional()
            .map_err(From::from)
            .map(|result| {
                result.map(|result| {
                    let (product_raw, (companies_package, company_raw, package_raw)) = result;
                    AvailablePackageForUser {
                        id: companies_package.id,
                        shipping_id: product_raw.id,
                        name: get_company_package_name(&company_raw.label, &package_raw.name),
                        logo: company_raw.logo,
                        price: product_raw.price,
                        currency: company_raw.currency,
                        shipping_variant: product_raw.shipping,
                        store_id: product_raw.store_id,
                        base_product_id: product_raw.base_product_id,
                    }
                })
            })
            .map_err(move |e: FailureError| {
                e.context(format!(
                    "Get available package for base product: {} with select company package id: {} failure.",
                    base_product_id_arg, package_id_arg
                ))
                .into()
            })
    }

    fn get_available_package_for_user_by_shipping_id(
        &self,
        shipping_id_arg: ShippingId,
        delivery_to: Option<Alpha3>,
    ) -> RepoResult<Option<AvailablePackageForUser>> {
        debug!("Get available package for shipping id: {}.", shipping_id_arg);

        let mut query = DslProducts::products
            .inner_join(
                DslCompaniesPackages::companies_packages
                    .inner_join(DslCompanies::companies)
                    .inner_join(DslPackages::packages),
            )
            .filter(DslProducts::id.eq(shipping_id_arg))
            .into_boxed();

        if let Some(delivery_to) = delivery_to {
            let pg_str = get_pg_str_json_array(vec![delivery_to.clone()]);
            query = query.filter(sql(format!("products.deliveries_to ?| {}", pg_str).as_ref()));
        };

        let query = query.order(DslCompanies::label);

        query
            .get_result::<(ProductsRaw, (CompaniesPackagesRaw, CompanyRaw, PackagesRaw))>(self.db_conn)
            .optional()
            .map_err(From::from)
            .map(|result| {
                result.map(|result| {
                    let (product_raw, (companies_package, company_raw, package_raw)) = result;
                    AvailablePackageForUser {
                        id: companies_package.id,
                        shipping_id: product_raw.id,
                        name: get_company_package_name(&company_raw.label, &package_raw.name),
                        logo: company_raw.logo,
                        price: product_raw.price,
                        currency: company_raw.currency,
                        shipping_variant: product_raw.shipping,
                        store_id: product_raw.store_id,
                        base_product_id: product_raw.base_product_id,
                    }
                })
            })
            .map_err(move |e: FailureError| {
                e.context(format!("Get available package for shipping id: {} failure.", shipping_id_arg))
                    .into()
            })
    }

    fn update(
        &self,
        base_product_id_arg: BaseProductId,
        company_package_id_arg: CompanyPackageId,
        payload: UpdateProducts,
    ) -> RepoResult<Products> {
        debug!("Updating products payload {:?}.", payload);
        let payload = payload.to_raw()?;
        self.execute_query(
            DslProducts::products
                .filter(DslProducts::base_product_id.eq(base_product_id_arg))
                .filter(DslProducts::company_package_id.eq(company_package_id_arg)),
        )
        .and_then(|products_: ProductsRaw| products_.to_products())
        .and_then(|product: Products| acl::check(&*self.acl, Resource::Products, Action::Update, self, Some(&product)))
        .and_then(|_| {
            let filter = DslProducts::products
                .filter(DslProducts::base_product_id.eq(base_product_id_arg))
                .filter(DslProducts::company_package_id.eq(company_package_id_arg));

            let query = diesel::update(filter).set(&payload);
            query.get_result::<ProductsRaw>(self.db_conn).map_err(From::from)
        })
        .and_then(|products_| products_.to_products())
        .map_err(|e: FailureError| e.context(format!("Updating products payload {:?} failed.", payload)).into())
    }

    fn delete(&self, base_product_id_arg: BaseProductId) -> RepoResult<Vec<Products>> {
        debug!("delete products {:?}.", base_product_id_arg);

        let filtered = DslProducts::products.filter(DslProducts::base_product_id.eq(base_product_id_arg));
        let query = diesel::delete(filtered);

        query
            .get_results(self.db_conn)
            .map_err(From::from)
            .and_then(|products_: Vec<ProductsRaw>| {
                let mut delete_products = vec![];
                for product in products_ {
                    let product = product.to_products()?;
                    acl::check(&*self.acl, Resource::Products, Action::Delete, self, Some(&product))?;
                    delete_products.push(product);
                }
                Ok(delete_products)
            })
            .map_err(|e: FailureError| {
                e.context(format!("Delete products with base product id {:?} failed.", base_product_id_arg))
                    .into()
            })
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, Products>
    for ProductsRepoImpl<'a, T>
{
    fn is_in_scope(&self, user_id_arg: UserId, scope: &Scope, obj: Option<&Products>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => {
                if let Some(obj) = obj {
                    Roles::roles
                        .filter(Roles::user_id.eq(user_id_arg))
                        .get_results::<UserRole>(self.db_conn)
                        .map_err(From::from)
                        .map(|user_roles_arg| {
                            user_roles_arg
                                .iter()
                                .any(|user_role_arg| user_role_arg.data.clone().map(|data| data == obj.store_id.0).unwrap_or_default())
                        })
                        .unwrap_or_else(|_: FailureError| false)
                } else {
                    false
                }
            }
        }
    }
}
