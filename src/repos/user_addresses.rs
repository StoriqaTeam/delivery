//! Repo for user_address table. UserAddress is an entity that connects
//! users and roles. I.e. this table is for user has-many roles
//! relationship

use diesel;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::Connection;
use failure::Error as FailureError;

use stq_types::UserId;

use repos::legacy_acl::*;

use super::acl;
use super::types::RepoResult;
use models::authorization::*;
use models::{NewUserAddress, UpdateUserAddress, UserAddress};
use schema::user_addresses::dsl::*;

/// UserAddresss repository for handling UserAddresss
pub trait UserAddressesRepo {
    /// Returns list of user_address for a specific user
    fn list_for_user(&self, user_id: UserId) -> RepoResult<Vec<UserAddress>>;

    /// Create a new user delivery address
    fn create(&self, payload: NewUserAddress) -> RepoResult<UserAddress>;

    /// Update a user delivery address
    fn update(&self, id: i32, payload: UpdateUserAddress) -> RepoResult<UserAddress>;

    /// Delete user delivery address
    fn delete(&self, id: i32) -> RepoResult<UserAddress>;
}

/// Implementation of UserAddresss trait
pub struct UserAddressesRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<Acl<Resource, Action, Scope, FailureError, UserAddress>>,
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> UserAddressesRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<Acl<Resource, Action, Scope, FailureError, UserAddress>>) -> Self {
        Self { db_conn, acl }
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> UserAddressesRepo
    for UserAddressesRepoImpl<'a, T>
{
    /// Returns list of user_address for a specific user
    fn list_for_user(&self, user_id_value: UserId) -> RepoResult<Vec<UserAddress>> {
        let query = user_addresses.filter(user_id.eq(user_id_value)).order(id.desc());
        query
            .get_results::<UserAddress>(self.db_conn)
            .map_err(From::from)
            .and_then(|addresses: Vec<UserAddress>| {
                for addres in &addresses {
                    acl::check(&*self.acl, Resource::UserAddresses, Action::Read, self, Some(&addres))?;
                }
                Ok(addresses)
            })
            .map_err(|e: FailureError| {
                e.context(format!("list of user_address for user {} error occured", user_id_value))
                    .into()
            })
    }

    /// Create a new user delivery address
    fn create(&self, payload: NewUserAddress) -> RepoResult<UserAddress> {
        let mut exist_query = user_addresses
            .filter(user_id.eq(payload.user_id))
            .filter(country.eq(payload.country.clone()))
            .filter(postal_code.eq(payload.postal_code.clone()))
            .into_boxed();

        if let Some(administrative_area_level_1_arg) = payload.administrative_area_level_1.clone() {
            exist_query = exist_query.filter(administrative_area_level_1.eq(administrative_area_level_1_arg));
        } else {
            exist_query = exist_query.filter(administrative_area_level_1.is_null());
        };
        if let Some(administrative_area_level_2_arg) = payload.administrative_area_level_2.clone() {
            exist_query = exist_query.filter(administrative_area_level_2.eq(administrative_area_level_2_arg));
        } else {
            exist_query = exist_query.filter(administrative_area_level_2.is_null())
        };
        if let Some(locality_arg) = payload.locality.clone() {
            exist_query = exist_query.filter(locality.eq(locality_arg));
        } else {
            exist_query = exist_query.filter(locality.is_null())
        };
        if let Some(political_arg) = payload.political.clone() {
            exist_query = exist_query.filter(political.eq(political_arg));
        } else {
            exist_query = exist_query.filter(political.is_null())
        };
        if let Some(route_arg) = payload.route.clone() {
            exist_query = exist_query.filter(route.eq(route_arg));
        } else {
            exist_query = exist_query.filter(route.is_null())
        };
        if let Some(street_number_arg) = payload.street_number.clone() {
            exist_query = exist_query.filter(street_number.eq(street_number_arg));
        } else {
            exist_query = exist_query.filter(street_number.is_null())
        };
        if let Some(address_arg) = payload.address.clone() {
            exist_query = exist_query.filter(address.eq(address_arg));
        } else {
            exist_query = exist_query.filter(address.is_null())
        };

        exist_query
            .get_result::<UserAddress>(self.db_conn)
            .optional()
            .map_err(From::from)
            .and_then(|user_address_arg| {
                if let Some(user_address_arg) = user_address_arg {
                    acl::check(&*self.acl, Resource::UserAddresses, Action::Create, self, Some(&user_address_arg))?;
                    Ok(user_address_arg)
                } else {
                    let query = diesel::insert_into(user_addresses).values(&payload);
                    query
                        .get_result(self.db_conn)
                        .map_err(From::from)
                        .and_then(|addres| {
                            acl::check(&*self.acl, Resource::UserAddresses, Action::Create, self, Some(&addres))?;
                            Ok(addres)
                        })
                        .and_then(|new_address| {
                            if new_address.is_priority {
                                // set all other addresses priority to false
                                let filter = user_addresses.filter(user_id.eq(new_address.user_id).and(id.ne(new_address.id)));
                                let query = diesel::update(filter).set(is_priority.eq(false));
                                let _ = query.get_result::<UserAddress>(self.db_conn);
                            }
                            Ok(new_address)
                        })
                }
            })
            .map_err(|e: FailureError| {
                e.context(format!("Create a new user delivery address {:?} error occured", payload))
                    .into()
            })
    }

    /// Update a user delivery address
    fn update(&self, id_arg: i32, payload: UpdateUserAddress) -> RepoResult<UserAddress> {
        let query = user_addresses.find(id_arg);

        query
            .get_result(self.db_conn)
            .map_err(From::from)
            .and_then(|addres: UserAddress| acl::check(&*self.acl, Resource::UserAddresses, Action::Update, self, Some(&addres)))
            .and_then(|_| {
                let filter = user_addresses.filter(id.eq(id_arg));

                let query = diesel::update(filter).set(&payload);
                query.get_result::<UserAddress>(self.db_conn).map_err(From::from)
            })
            .and_then(|updated_address| {
                if let Some(is_priority_arg) = payload.is_priority {
                    if is_priority_arg {
                        // set all other addresses priority to false
                        let filter = user_addresses.filter(user_id.eq(updated_address.user_id).and(id.ne(updated_address.id)));
                        let query = diesel::update(filter).set(is_priority.eq(false));
                        let _ = query.get_result::<UserAddress>(self.db_conn);
                    }
                }
                Ok(updated_address)
            })
            .map_err(|e: FailureError| {
                e.context(format!("Update address {} with payload {:?} error occured", id_arg, payload))
                    .into()
            })
    }

    /// Delete user delivery address
    fn delete(&self, id_arg: i32) -> RepoResult<UserAddress> {
        let query = user_addresses.find(id_arg);

        query
            .get_result(self.db_conn)
            .map_err(From::from)
            .and_then(|addres: UserAddress| acl::check(&*self.acl, Resource::UserAddresses, Action::Delete, self, Some(&addres)))
            .and_then(|_| {
                let filtered = user_addresses.filter(id.eq(id_arg));
                let query = diesel::delete(filtered);
                query.get_result(self.db_conn).map_err(From::from)
            })
            .map_err(|e: FailureError| e.context(format!("Delete delivery address {} error occured", id_arg)).into())
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, UserAddress>
    for UserAddressesRepoImpl<'a, T>
{
    fn is_in_scope(&self, user_id_arg: UserId, scope: &Scope, obj: Option<&UserAddress>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => {
                if let Some(addres) = obj {
                    addres.user_id == user_id_arg
                } else {
                    false
                }
            }
        }
    }
}
