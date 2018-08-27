//! Repo DeliveryFrom table.

use diesel;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::LoadQuery;
use diesel::query_dsl::RunQueryDsl;
use diesel::Connection;

use failure::Error as FailureError;
use failure::Fail;

use stq_static_resources::DeliveryCompany;
use stq_types::UserId;

use super::types::RepoResult;
use models::authorization::*;
use models::company::{DeliveryFrom, NewDeliveryFrom, UpdateDeliveryFrom};
use repos::legacy_acl::*;
use repos::*;
use schema::delivery_from::dsl::*;

/// DeliveryFrom repository for handling DeliveryFrom
pub trait DeliveryFromRepo {
    /// Create a new delivery
    fn create(&self, payload: NewDeliveryFrom) -> RepoResult<DeliveryFrom>;

    /// Returns list of deliveries supported by the company, limited by `from` parameter
    fn list_by_company(&self, from: DeliveryCompany) -> RepoResult<Vec<DeliveryFrom>>;

    /// Update a delivery
    fn update(&self, payload: UpdateDeliveryFrom) -> RepoResult<DeliveryFrom>;

    /// Delete a delivery
    fn delete(&self, company_id: DeliveryCompany, country: String) -> RepoResult<DeliveryFrom>;
}

/// Implementation of UserRoles trait
pub struct DeliveryFromRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<Acl<Resource, Action, Scope, FailureError, DeliveryFrom>>,
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> DeliveryFromRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<Acl<Resource, Action, Scope, FailureError, DeliveryFrom>>) -> Self {
        Self { db_conn, acl }
    }

    fn execute_query<Ty: Send + 'static, U: LoadQuery<T, Ty> + Send + 'static>(&self, query: U) -> RepoResult<Ty> {
        query.get_result::<Ty>(self.db_conn).map_err(From::from)
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> DeliveryFromRepo
    for DeliveryFromRepoImpl<'a, T>
{
    fn create(&self, payload: NewDeliveryFrom) -> RepoResult<DeliveryFrom> {
        debug!("create new delivery_from {:?}.", payload);
        let query = diesel::insert_into(delivery_from).values(&payload);
        query
            .get_result::<DeliveryFrom>(self.db_conn)
            .map_err(From::from)
            .and_then(|delivery| {
                acl::check(&*self.acl, Resource::DeliveryFrom, Action::Create, self, Some(&delivery)).and_then(|_| Ok(delivery))
            })
            .map_err(|e: FailureError| e.context(format!("create new delivery_from {:?}.", payload)).into())
    }

    fn list_by_company(&self, from: DeliveryCompany) -> RepoResult<Vec<DeliveryFrom>> {
        debug!("Find in delivery_from with ids from {:?}.", from);
        let query = delivery_from.filter(company_id.eq(from.clone()));

        query
            .get_results(self.db_conn)
            .map_err(From::from)
            .and_then(|deliveries_res: Vec<DeliveryFrom>| {
                for delivery in &deliveries_res {
                    acl::check(&*self.acl, Resource::DeliveryFrom, Action::Read, self, Some(&delivery))?;
                }
                Ok(deliveries_res)
            })
            .map_err(|e: FailureError| {
                e.context(format!("Find in delivery_from with ids from {:?} error occured", from))
                    .into()
            })
    }

    fn update(&self, payload: UpdateDeliveryFrom) -> RepoResult<DeliveryFrom> {
        debug!("Updating delivery_from payload {:?}.", payload);
        self.execute_query(
            delivery_from
                .filter(company_id.eq(payload.company_id.clone()))
                .filter(country.eq(payload.country.clone())),
        ).and_then(|delivery: DeliveryFrom| acl::check(&*self.acl, Resource::DeliveryFrom, Action::Update, self, Some(&delivery)))
            .and_then(|_| {
                let filtered = delivery_from
                    .filter(company_id.eq(payload.company_id.clone()))
                    .filter(country.eq(payload.country.clone()));

                let query = diesel::update(filtered).set(restriction_name.eq(&payload.restriction_name));
                query.get_result::<DeliveryFrom>(self.db_conn).map_err(From::from)
            })
            .map_err(|e: FailureError| e.context(format!("Updating delivery_from payload {:?} failed.", payload)).into())
    }

    fn delete(&self, company_id_: DeliveryCompany, country_: String) -> RepoResult<DeliveryFrom> {
        debug!("delete delivery_from company_id: {} country: {}.", company_id_, country_);

        acl::check(&*self.acl, Resource::DeliveryFrom, Action::Delete, self, None)?;
        let filter = delivery_from
            .filter(company_id.eq(company_id_.clone()))
            .filter(country.eq(country_.clone()));
        let query = diesel::delete(filter);
        query.get_result::<DeliveryFrom>(self.db_conn).map_err(move |e| {
            e.context(format!("delete delivery_from company_id: {} country: {}.", company_id_, country_))
                .into()
        })
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, DeliveryFrom>
    for DeliveryFromRepoImpl<'a, T>
{
    fn is_in_scope(&self, _user_id: UserId, scope: &Scope, _obj: Option<&DeliveryFrom>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => false,
        }
    }
}
