//! REPO Products table. Products is an entity that
//! contains info about local shipping of base_product.

use diesel;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::LoadQuery;
use diesel::query_dsl::RunQueryDsl;
use diesel::Connection;
use failure::Error as FailureError;

use stq_types::{BaseProductId, UserId};

use models::authorization::*;
use models::products::{NewProducts, Products, ProductsRaw, UpdateProducts};
use models::roles::UserRole;
use repos::legacy_acl::*;
use repos::types::RepoResult;
use repos::*;
use schema::products::dsl::*;
use schema::roles::dsl as Roles;

/// Products repository for handling Products
pub trait ProductsRepo {
    /// Create a new products
    fn create(&self, payload: NewProducts) -> RepoResult<Products>;

    /// Get a products
    fn get_by_base_product_id(&self, base_product_id: BaseProductId) -> RepoResult<Vec<Products>>;

    /// Update a products
    fn update(&self, base_product_id_arg: BaseProductId, company_package_id: i32, payload: UpdateProducts) -> RepoResult<Products>;

    /// Delete a products
    fn delete(&self, base_product_id_arg: BaseProductId) -> RepoResult<Products>;
}

pub struct ProductsRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<Acl<Resource, Action, Scope, FailureError, Products>>,
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> ProductsRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<Acl<Resource, Action, Scope, FailureError, Products>>) -> Self {
        Self { db_conn, acl }
    }

    fn execute_query<Ty: Send + 'static, U: LoadQuery<T, Ty> + Send + 'static>(&self, query: U) -> RepoResult<Ty> {
        query.get_result::<Ty>(self.db_conn).map_err(From::from)
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> ProductsRepo for ProductsRepoImpl<'a, T> {
    fn create(&self, payload: NewProducts) -> RepoResult<Products> {
        debug!("create new products {:?}.", payload);
        let payload = payload.to_raw()?;
        let query = diesel::insert_into(products).values(&payload);
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

    fn get_by_base_product_id(&self, base_product_id_arg: BaseProductId) -> RepoResult<Vec<Products>> {
        debug!("get products by base_product_id {:?}.", base_product_id_arg);
        let query = products.filter(base_product_id.eq(base_product_id_arg));

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

    fn update(&self, base_product_id_arg: BaseProductId, company_package_id_arg: i32, payload: UpdateProducts) -> RepoResult<Products> {
        debug!("Updating products payload {:?}.", payload);
        let payload = payload.to_raw()?;
        self.execute_query(
            products
                .filter(base_product_id.eq(base_product_id_arg))
                .filter(company_package_id.eq(company_package_id_arg)),
        ).and_then(|products_: ProductsRaw| products_.to_products())
            .and_then(|product: Products| acl::check(&*self.acl, Resource::Products, Action::Update, self, Some(&product)))
            .and_then(|_| {
                let filter = products
                    .filter(base_product_id.eq(base_product_id_arg))
                    .filter(company_package_id.eq(company_package_id_arg));

                let query = diesel::update(filter).set(&payload);
                query.get_result::<ProductsRaw>(self.db_conn).map_err(From::from)
            })
            .and_then(|products_| products_.to_products())
            .map_err(|e: FailureError| e.context(format!("Updating products payload {:?} failed.", payload)).into())
    }

    fn delete(&self, base_product_id_arg: BaseProductId) -> RepoResult<Products> {
        debug!("delete products {:?}.", base_product_id_arg);
        self.execute_query(products.filter(base_product_id.eq(base_product_id_arg)))
            .and_then(|products_: ProductsRaw| products_.to_products())
            .and_then(|product: Products| acl::check(&*self.acl, Resource::Products, Action::Delete, self, Some(&product)))
            .and_then(|_| {
                let filter = products.filter(base_product_id.eq(base_product_id_arg));

                let query = diesel::delete(filter);
                query.get_result::<ProductsRaw>(self.db_conn).map_err(From::from)
            })
            .and_then(|products_| products_.to_products())
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
