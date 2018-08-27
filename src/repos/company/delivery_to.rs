//! Repo DeliveryTo table.

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

use models::authorization::*;
use models::company::{DeliveryTo, DeliveryToRaw, NewDeliveryTo, UpdateDeliveryTo};
use repos::legacy_acl::*;
use repos::types::RepoResult;
use repos::*;

use schema::delivery_to::dsl::*;

/// DeliveryTo repository for handling DeliveryTo
pub trait DeliveryToRepo {
    /// Create a new delivery
    fn create(&self, payload: NewDeliveryTo) -> RepoResult<DeliveryTo>;

    /// Returns list of deliveries supported by the company, limited by `from` parameter
    fn list_by_company(&self, from: DeliveryCompany) -> RepoResult<Vec<DeliveryTo>>;

    /// Returns list of deliveries supported by the country, limited by `from` parameter
    fn list_by_country(&self, from: String) -> RepoResult<Vec<DeliveryTo>>;

    /// Update a delivery
    fn update(&self, payload: UpdateDeliveryTo) -> RepoResult<DeliveryTo>;

    /// Delete a delivery
    fn delete(&self, company_id: DeliveryCompany, country: String) -> RepoResult<DeliveryTo>;
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

    fn list_by_company(&self, from: DeliveryCompany) -> RepoResult<Vec<DeliveryTo>> {
        debug!("Find in delivery_to with ids from {:?}.", from);
        let query = delivery_to.filter(company_id.eq(from.clone()));

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
                e.context(format!("Find in delivery_to with ids from {:?} error occured", from))
                    .into()
            })
    }

    fn list_by_country(&self, from: String) -> RepoResult<Vec<DeliveryTo>> {
        debug!("Find in delivery_to with ids from {}.", from);
        let query = delivery_to.filter(country.eq(from.clone())).order(country);

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
                e.context(format!("Find in delivery_to with ids from {} error occured", from))
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

    fn delete(&self, company_id_: DeliveryCompany, country_: String) -> RepoResult<DeliveryTo> {
        debug!("delete delivery company_id: {} country: {}.", company_id_, country_);

        acl::check(&*self.acl, Resource::DeliveryTo, Action::Delete, self, None)?;
        let filter = delivery_to
            .filter(company_id.eq(company_id_.clone()))
            .filter(country.eq(country_.clone()));
        let query = diesel::delete(filter);
        query
            .get_result::<DeliveryToRaw>(self.db_conn)
            .map_err(move |e| {
                e.context(format!("delete delivery company_id: {} country: {}.", company_id_, country_))
                    .into()
            })
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
