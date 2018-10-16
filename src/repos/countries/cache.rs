//! CountryCache is a module that caches received from db information about user and his categories
use failure::{Error as FailureError, err_msg};
use r2d2::Pool;
use r2d2_redis::RedisConnectionManager;
use serde_json;
use stq_cache::cache::{Cache, redis::RedisCache};

use models::Country;

#[derive(Clone)]
pub struct CountryCacheImpl {
    redis_pool: Pool<RedisConnectionManager>,
}

impl CountryCacheImpl {
    const KEY: &'static str = "country";
    const TTL_SECONDS: u32 = 600;

    pub fn new(redis_pool: Pool<RedisConnectionManager>) -> Self {
        CountryCacheImpl {
            redis_pool
        }
    }

    pub fn get(&self) -> Option<Country> {
        debug!("Getting country from CountryCache");

        let result = self.with_cache(|cache| cache.get(CountryCacheImpl::KEY))
            .and_then(|res| res.map_err(|err| FailureError::from(err).context("Failed to get country from CountryCache").into()))
            .and_then(|value_json|
                match value_json {
                    None => Ok(None),
                    Some(json) => serde_json::from_str(&json)
                        .map(Some)
                        .map_err(|e| FailureError::from(e).context("Failed to deserialize value returned from CountryCache").into())
                });

        result.unwrap_or_else(|err| { error!("{}", err); None })
    }

    pub fn remove(&self) {
        debug!("Removing country from CountryCache");

        let result = self.with_cache(|cache| cache.remove(CountryCacheImpl::KEY))
            .and_then(|res| res
                .map(|_| ())
                .map_err(|err| FailureError::from(err).context("Failed to remove country from CountryCache").into()));
        
        result.unwrap_or_else(|err| { error!("{}", err); })
    }

    pub fn set(&self, country: &Country) {
        debug!("Setting country in CountryCache to '{}'", country.label);
        
        let country_json = serde_json::to_string(country).unwrap();
        let result =
            self.with_cache(|cache| cache.set(CountryCacheImpl::KEY, country_json.as_str(), CountryCacheImpl::TTL_SECONDS))
                .and_then(|res| res.map_err(|e| FailureError::from(e).context("Failed to set country in CountryCache").into()));

        result.unwrap_or_else(|err| { error!("{}", err); })
    }

    fn with_cache<T, F>(&self, f: F) -> Result<T, FailureError>
        where F: Fn(&RedisCache) -> T
    {
        debug!("Getting Redis connection for CountryCache");

        self.redis_pool.try_get().ok_or(err_msg("Failed to get Redis connection for CountryCache"))
            .map(|conn| {
                let cache = RedisCache::new(&conn, "country");
                f(&cache)
            })
    }
}
