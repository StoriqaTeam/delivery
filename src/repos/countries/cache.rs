//! CountryCache is a module that caches received from db information about user and his categories
use failure::Fail;
use stq_cache::cache::CacheSingle;

use models::Country;

pub struct CountryCacheImpl<C>
where
    C: CacheSingle<Country>,
{
    cache: C,
}

impl<C> CountryCacheImpl<C>
where
    C: CacheSingle<Country>,
{
    pub fn new(cache: C) -> Self {
        CountryCacheImpl { cache }
    }

    pub fn get(&self) -> Option<Country> {
        debug!("Getting country from CountryCache");

        self.cache.get().unwrap_or_else(|err| {
            error!("{}", err.context("Failed to get country from CountryCache"));
            None
        })
    }

    pub fn remove(&self) -> bool {
        debug!("Removing country from CountryCache");

        self.cache.remove().unwrap_or_else(|err| {
            error!("{}", err.context("Failed to remove country from CountryCache"));
            false
        })
    }

    pub fn set(&self, country: &Country) {
        debug!("Setting country in CountryCache");

        self.cache.set(country.clone()).unwrap_or_else(|err| {
            error!("{}", err.context("Failed to set country in CountryCache"));
        })
    }
}
