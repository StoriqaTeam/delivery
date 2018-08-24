//! Repo DeliveryTo table. DeliveryTo is an entity that
//! describes the limits of the delivery company on
//! the dimensions of the goods.

use super::types::RepoResult;
use models::company::{DeliveryTo, DeliveryToRaw, NewDeliveryTo, OldDeliveryTo, UpdateDeliveryTo};

use diesel;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::LoadQuery;
use diesel::query_dsl::RunQueryDsl;
use diesel::Connection;

use models::authorization::*;
use stq_static_resources::DeliveryCompany;

use repos::legacy_acl::*;
use repos::*;

use failure::Error as FailureError;
use failure::Fail;

use stq_types::UserId;

use schema::delivery_to::dsl::*;

/// DeliveryTo repository for handling DeliveryTo
pub trait DeliveryToRepo {
    /// Create a new delivery
    fn create(&self, payload: NewDeliveryTo) -> RepoResult<DeliveryTo>;

    /// Returns list of deliveries supported by the company, limited by `from` and `count` parameters
    fn list_by_company(&self, from: DeliveryCompany, count: i32) -> RepoResult<Vec<DeliveryTo>>;

    /// Returns list of deliveries supported by the country, limited by `from` and `count` parameters
    fn list_by_country(&self, from: String, count: i32) -> RepoResult<Vec<DeliveryTo>>;

    /// Update a delivery
    fn update(&self, payload: UpdateDeliveryTo) -> RepoResult<DeliveryTo>;

    /// Delete a delivery
    fn delete(&self, payload: OldDeliveryTo) -> RepoResult<DeliveryTo>;
}

/// Implementation of UserRoles trait
pub struct DeliveryToRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<Acl<Resource, Action, Scope, FailureError, DeliveryTo>>,
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> DeliveryToRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<Acl<Resource, Action, Scope, FailureError, DeliveryTo>>) -> Self {
        Self { db_conn, acl }
    }

    fn execute_query<Ty: Send + 'static, U: LoadQuery<T, Ty> + Send + 'static>(&self, query: U) -> RepoResult<Ty> {
        query.get_result::<Ty>(self.db_conn).map_err(From::from)
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> DeliveryToRepo for DeliveryToRepoImpl<'a, T> {
    fn create(&self, payload: NewDeliveryTo) -> RepoResult<DeliveryTo> {
        debug!("create new delivery {:?}.", payload);
        let payload = payload.to_raw()?;
        let query = diesel::insert_into(delivery_to).values(&payload);
        query
            .get_result::<DeliveryToRaw>(self.db_conn)
            .map_err(From::from)
            .and_then(DeliveryTo::from_raw)
            .and_then(|delivery| {
                acl::check(&*self.acl, Resource::DeliveryTo, Action::Create, self, Some(&delivery)).and_then(|_| Ok(delivery))
            })
            .map_err(|e: FailureError| e.context(format!("create new delivery {:?}.", payload)).into())
    }

    fn list_by_company(&self, from: DeliveryCompany, count: i32) -> RepoResult<Vec<DeliveryTo>> {
        debug!("Find in delivery_to with ids from {:?} count {}.", from, count);
        let query = delivery_to.filter(company_id.eq(from.clone())).limit(count.into());

        query
            .get_results(self.db_conn)
            .map_err(From::from)
            .and_then(|deliveries_raw: Vec<DeliveryToRaw>| {
                deliveries_raw
                    .into_iter()
                    .map(|delivery_raw| DeliveryTo::from_raw(delivery_raw))
                    .collect()
            })
            .and_then(|deliveries_res: Vec<DeliveryTo>| {
                for delivery in &deliveries_res {
                    acl::check(&*self.acl, Resource::DeliveryTo, Action::Read, self, Some(&delivery))?;
                }
                Ok(deliveries_res)
            })
            .map_err(|e: FailureError| {
                e.context(format!(
                    "Find in delivery_to with ids from {:?} count {} error occured",
                    from, count
                )).into()
            })
    }

    fn list_by_country(&self, from: String, count: i32) -> RepoResult<Vec<DeliveryTo>> {
        debug!("Find in delivery_to with ids from {} count {}.", from, count);
        let query = delivery_to.filter(country.eq(from.clone())).order(country).limit(count.into());

        query
            .get_results(self.db_conn)
            .map_err(From::from)
            .and_then(|deliveries_raw: Vec<DeliveryToRaw>| {
                deliveries_raw
                    .into_iter()
                    .map(|delivery_raw| DeliveryTo::from_raw(delivery_raw))
                    .collect()
            })
            .and_then(|deliveries_res: Vec<DeliveryTo>| {
                for delivery in &deliveries_res {
                    acl::check(&*self.acl, Resource::DeliveryTo, Action::Read, self, Some(&delivery))?;
                }
                Ok(deliveries_res)
            })
            .map_err(|e: FailureError| {
                e.context(format!("Find in delivery_to with ids from {} count {} error occured", from, count))
                    .into()
            })
    }

    fn update(&self, payload: UpdateDeliveryTo) -> RepoResult<DeliveryTo> {
        debug!("Updating delivery payload {:?}.", payload);
        let payload = payload.to_raw()?;
        self.execute_query(
            delivery_to
                .filter(company_id.eq(payload.company_id.clone()))
                .filter(country.eq(payload.country.clone())),
        ).and_then(DeliveryTo::from_raw)
            .and_then(|delivery: DeliveryTo| acl::check(&*self.acl, Resource::DeliveryTo, Action::Update, self, Some(&delivery)))
            .and_then(|_| {
                let filtered = delivery_to
                    .filter(company_id.eq(payload.company_id.clone()))
                    .filter(country.eq(payload.country.clone()));

                let query = diesel::update(filtered).set(additional_info.eq(&payload.additional_info));
                query
                    .get_result::<DeliveryToRaw>(self.db_conn)
                    .map_err(From::from)
                    .and_then(DeliveryTo::from_raw)
            })
            .map_err(|e: FailureError| e.context(format!("Updating delivery_to payload {:?} failed.", payload)).into())
    }

    fn delete(&self, payload: OldDeliveryTo) -> RepoResult<DeliveryTo> {
        debug!("delete delivery {:?}.", payload);
        let OldDeliveryTo {
            company_id: company_id_,
            country: country_,
        } = payload.clone();
        acl::check(&*self.acl, Resource::DeliveryTo, Action::Delete, self, None)?;
        let filter = delivery_to.filter(company_id.eq(company_id_)).filter(country.eq(country_));
        let query = diesel::delete(filter);
        query
            .get_result::<DeliveryToRaw>(self.db_conn)
            .map_err(move |e| e.context(format!("delete delivery {:?}.", payload)).into())
            .and_then(DeliveryTo::from_raw)
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, DeliveryTo>
    for DeliveryToRepoImpl<'a, T>
{
    fn is_in_scope(&self, _user_id: UserId, scope: &Scope, _obj: Option<&DeliveryTo>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => false,
        }
    }
}
