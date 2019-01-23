extern crate delivery_lib as lib;
extern crate futures;
extern crate hyper;
extern crate rand;
extern crate serde_json;
extern crate stq_http;
extern crate stq_static_resources;
extern crate stq_types;
extern crate tokio_core;

mod common;

mod integration_companies_packages_test;
mod integration_companies_test;
mod integration_countries_test;
mod integration_packages_test;
mod integration_products_test;
mod integration_user_addresses_test;
