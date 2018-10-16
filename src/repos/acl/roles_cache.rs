//! RolesCache is a module that caches received from db information about user and his roles

use failure::{Error as FailureError, err_msg};
use r2d2::Pool;
use r2d2_redis::RedisConnectionManager;
use serde_json;
use stq_cache::cache::{Cache, redis::RedisCache};
use stq_types::{DeliveryRole, UserId};

#[derive(Clone)]
pub struct RolesCacheImpl {
    redis_pool: Pool<RedisConnectionManager>,
}

impl RolesCacheImpl {
    const TTL_SECONDS: u32 = 600;

    pub fn new(redis_pool: Pool<RedisConnectionManager>) -> Self {
        RolesCacheImpl {
            redis_pool
        }
    }

    pub fn get(&self, user_id: UserId) -> Option<Vec<DeliveryRole>> {
        debug!("Getting roles from RolesCache at key '{}'", user_id);

        let result = self.with_cache(|cache| cache.get(user_id.to_string().as_str()))
            .and_then(|res| res.map_err(|err| FailureError::from(err).context("Failed to get roles from RolesCache").into()))
            .and_then(|value_json|
                match value_json {
                    None => Ok(None),
                    Some(json) => serde_json::from_str(&json)
                        .map(Some)
                        .map_err(|e| {
                            let msg = format!("Failed to deserialize value returned from RolesCache at key '{}'", user_id);
                            FailureError::from(e).context(msg).into()
                        })
                });

        result.unwrap_or_else(|err| { error!("{}", err); None })
    }

    pub fn remove(&self, user_id: UserId) {
        debug!("Removing roles from RolesCache at key '{}'", user_id);

        let result = self.with_cache(|cache| cache.remove(user_id.to_string().as_str()))
            .and_then(|res|
                res
                    .map(|_| ())
                    .map_err(|err| FailureError::from(err).context("Failed to remove roles from RolesCache").into()));
        
        result.unwrap_or_else(|err| { error!("{}", err); })
    }

    pub fn set(&self, user_id: UserId, roles: &[DeliveryRole]) {
        debug!("Setting roles in RolesCache at key '{}' to '{:?}'", user_id, roles);

        let roles_json = serde_json::to_string(roles).unwrap();
        let result =
            self.with_cache(|cache|
                cache.set(
                    user_id.to_string().as_str(),
                    roles_json.as_str(),
                    RolesCacheImpl::TTL_SECONDS)
            ).and_then(|res| res.map_err(|e| FailureError::from(e).context("Failed to set roles in RolesCache").into()));

        result.unwrap_or_else(|err| { error!("{}", err); })
    }

    fn with_cache<T, F>(&self, f: F) -> Result<T, FailureError>
        where F: Fn(&RedisCache) -> T
    {
        debug!("Getting Redis connection for RolesCache");

        self.redis_pool.try_get().ok_or(err_msg("Failed to get Redis connection for RolesCache"))
            .map(|conn| {
                let cache = RedisCache::new(&conn, "roles");
                f(&cache)
            })
    }
}
