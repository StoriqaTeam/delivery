//! Repo for shipping_rates table. ShippingRates contains rates for every available shipping direction for company-package

use diesel::connection::AnsiTransactionManager;
use diesel::pg::expression::dsl::any;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::Connection;
use failure::Error as FailureError;

use stq_types::{Alpha3, CompanyPackageId, UserId};

use repos::legacy_acl::*;

use super::acl;
use super::types::RepoResult;
use extras::option;
use models::authorization::*;
use models::{ShippingRates, ShippingRatesRaw};
use schema::shipping_rates::dsl as DslShippingRates;

/// Repository for static shipping rates
pub trait ShippingRatesRepo {
    fn get_multiple_rates(
        &self,
        company_package_id: CompanyPackageId,
        delivery_from: Alpha3,
        deliveries_to: Vec<Alpha3>,
    ) -> RepoResult<Vec<ShippingRates>>;

    fn get_rates(
        &self,
        company_package_id: CompanyPackageId,
        delivery_from: Alpha3,
        delivery_to: Alpha3,
    ) -> RepoResult<Option<ShippingRates>>;
}

pub struct ShippingRatesRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<Acl<Resource, Action, Scope, FailureError, ()>>,
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> ShippingRatesRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<Acl<Resource, Action, Scope, FailureError, ()>>) -> Self {
        Self { db_conn, acl }
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> ShippingRatesRepo
    for ShippingRatesRepoImpl<'a, T>
{
    fn get_multiple_rates(
        &self,
        company_package_id: CompanyPackageId,
        delivery_from: Alpha3,
        deliveries_to: Vec<Alpha3>,
    ) -> RepoResult<Vec<ShippingRates>> {
        acl::check(&*self.acl, Resource::ShippingRates, Action::Read, self, None)?;

        let query = DslShippingRates::shipping_rates
            .filter(
                DslShippingRates::company_package_id
                    .eq(company_package_id)
                    .and(DslShippingRates::from_alpha3.eq(delivery_from.clone()))
                    .and(DslShippingRates::to_alpha3.eq(any(deliveries_to.clone()))),
            ).order(DslShippingRates::id.desc());

        query
            .get_results::<ShippingRatesRaw>(self.db_conn)
            .map_err(FailureError::from)
            .and_then(|rates| rates.into_iter().map(ShippingRatesRaw::to_model).collect::<Result<Vec<_>, _>>())
            .map_err(|e| {
                e.context(format!(
                    "error occurred in get_multiple_rates for CompanyPackage with id = {}, {} -> {:?}",
                    company_package_id, delivery_from, deliveries_to,
                )).into()
            })
    }

    fn get_rates(
        &self,
        company_package_id: CompanyPackageId,
        delivery_from: Alpha3,
        delivery_to: Alpha3,
    ) -> RepoResult<Option<ShippingRates>> {
        acl::check(&*self.acl, Resource::ShippingRates, Action::Read, self, None)?;

        let query = DslShippingRates::shipping_rates
            .filter(
                DslShippingRates::company_package_id
                    .eq(company_package_id)
                    .and(DslShippingRates::from_alpha3.eq(delivery_from.clone()))
                    .and(DslShippingRates::to_alpha3.eq(delivery_to.clone())),
            ).order(DslShippingRates::id.desc());

        query
            .get_result::<ShippingRatesRaw>(self.db_conn)
            .optional()
            .map_err(FailureError::from)
            .and_then(|rates| option::transpose(rates.map(ShippingRatesRaw::to_model)))
            .map_err(|e| {
                e.context(format!(
                    "error occurred in get_rates for CompanyPackage with id = {}, {} -> {}",
                    company_package_id, delivery_from, delivery_to,
                )).into()
            })
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, ()>
    for ShippingRatesRepoImpl<'a, T>
{
    fn is_in_scope(&self, _user_id_arg: UserId, _scope: &Scope, _obj: Option<&()>) -> bool {
        true
    }
}
