#![allow(proc_macro_derive_resolution_fallback)]
extern crate base64;
extern crate chrono;
extern crate config as config_crate;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate failure;
extern crate futures;
extern crate futures_cpupool;
extern crate hyper;
extern crate hyper_tls;
extern crate jsonwebtoken;
#[macro_use]
extern crate log;
extern crate r2d2;
extern crate r2d2_redis;
extern crate rand;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate mime;
extern crate serde_json;
extern crate sha3;
extern crate tokio_core;
extern crate tokio_signal;
extern crate uuid;
extern crate validator;
#[macro_use]
extern crate validator_derive;
#[macro_use]
extern crate sentry;

extern crate stq_cache;
#[macro_use]
extern crate stq_http;
extern crate stq_logging;
extern crate stq_router;
extern crate stq_static_resources;
#[macro_use]
extern crate stq_diesel_macro_derive;
extern crate stq_types;

pub mod config;
pub mod controller;
pub mod errors;
pub mod extras;
pub mod models;
pub mod repos;
pub mod schema;
pub mod sentry_integration;
pub mod services;

use std::process;
use std::sync::Arc;
use std::time::Duration;

use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use futures::future;
use futures::prelude::*;
use futures_cpupool::CpuPool;
use hyper::server::Http;
use r2d2_redis::RedisConnectionManager;
use stq_cache::cache::{redis::RedisCache, Cache, NullCache, TypedCache};
use stq_http::controller::Application;
use tokio_core::reactor::Core;

use controller::context::StaticContext;
use repos::acl::RolesCacheImpl;
use repos::countries::CountryCacheImpl;
use repos::repo_factory::ReposFactoryImpl;

/// Starts new web service from provided `Config`
pub fn start_server<F: FnOnce() + 'static>(config: config::Config, port: Option<i32>, callback: F) {
    let thread_count = config.server.thread_count;
    let cpu_pool = CpuPool::new(thread_count);

    // Prepare reactor
    let mut core = Core::new().expect("Unexpected error creating event loop core");
    let handle = Arc::new(core.handle());

    // Prepare database pool
    let database_url: String = config.server.database.parse().expect("Database URL must be set in configuration");
    let db_manager = ConnectionManager::<PgConnection>::new(database_url);
    let db_pool = r2d2::Pool::builder()
        .build(db_manager)
        .expect("Failed to create DB connection pool");

    // Prepare server
    let address = {
        let port = port.as_ref().unwrap_or(&config.server.port);
        format!("{}:{}", config.server.host, port).parse().expect("Could not parse address")
    };

    let (country_cache, roles_cache) = match &config.server.redis {
        Some(redis_url) => {
            // Prepare Redis pool
            let redis_url: String = redis_url.parse().expect("Redis URL must be set in configuration");
            let redis_manager = RedisConnectionManager::new(redis_url.as_ref()).expect("Failed to create Redis connection manager");
            let redis_pool = r2d2::Pool::builder()
                .build(redis_manager)
                .expect("Failed to create Redis connection pool");

            let ttl = Duration::from_secs(config.server.cache_ttl_sec);

            let country_cache_backend = Box::new(TypedCache::new(
                RedisCache::new(redis_pool.clone(), "country".to_string()).with_ttl(ttl),
            )) as Box<dyn Cache<_, Error = _> + Send + Sync>;
            let country_cache = CountryCacheImpl::new(country_cache_backend);

            let roles_cache_backend = Box::new(TypedCache::new(
                RedisCache::new(redis_pool.clone(), "roles".to_string()).with_ttl(ttl),
            )) as Box<dyn Cache<_, Error = _> + Send + Sync>;
            let roles_cache = RolesCacheImpl::new(roles_cache_backend);

            (country_cache, roles_cache)
        }
        None => (
            CountryCacheImpl::new(Box::new(NullCache::new()) as Box<_>),
            RolesCacheImpl::new(Box::new(NullCache::new()) as Box<_>),
        ),
    };

    // Repo factory
    let repo_factory = ReposFactoryImpl::new(country_cache, roles_cache);

    let client = stq_http::client::Client::new(&config.to_http_config(), &handle);
    let client_handle = client.handle();
    let client_stream = client.stream();
    handle.spawn(client_stream.for_each(|_| Ok(())));

    let context = StaticContext::new(db_pool, cpu_pool, client_handle, Arc::new(config), repo_factory);

    let serve = Http::new()
        .serve_addr_handle(&address, &*handle, move || {
            // Prepare application
            let controller = controller::ControllerImpl::new(context.clone());
            let app = Application::<errors::Error>::new(controller);

            Ok(app)
        }).unwrap_or_else(|reason| {
            eprintln!("Http Server Initialization Error: {}", reason);
            process::exit(1);
        });

    handle.spawn(
        serve
            .for_each({
                let handle = handle.clone();
                move |conn| {
                    handle.spawn(conn.map(|_| ()).map_err(|why| eprintln!("Server Error: {:?}", why)));
                    Ok(())
                }
            }).map_err(|_| ()),
    );

    info!("Listening on http://{}, threads: {}", address, thread_count);
    handle.spawn_fn(move || {
        callback();
        future::ok(())
    });

    core.run(tokio_signal::ctrl_c().flatten_stream().take(1u64).for_each(|()| {
        info!("Ctrl+C received. Exit");
        Ok(())
    })).unwrap();
}
