extern crate futures;
extern crate hyper;
extern crate rand;
extern crate serde_json;
extern crate stq_http;
extern crate stq_types;
extern crate tokio_core;

extern crate delivery_lib as lib;

use lib::models::*;
use stq_types::*;

use self::futures::prelude::*;
use self::rand::Rng;
use self::stq_http::client::{self, Client as HttpClient, ClientHandle as HttpClientHandle, Config as HttpConfig};
use self::tokio_core::reactor::Core;
use hyper::Method;
use std::sync::mpsc::channel;
use std::thread;

pub fn setup() -> String {
    let (tx, rx) = channel::<bool>();
    let mut rng = rand::thread_rng();
    let port = rng.gen_range(50000, 60000);
    thread::spawn({
        let tx = tx.clone();
        move || {
            let config = lib::config::Config::new().expect("Can't load app config!");
            lib::start_server(config, Some(port), move || {
                let _ = tx.send(true);
            });
        }
    });
    rx.recv().unwrap();

    format!("http://localhost:{}", port)
}

pub fn make_utils() -> (Core, HttpClientHandle) {
    let core = Core::new().expect("Unexpected error creating event loop core");
    let client = HttpClient::new(
        &HttpConfig {
            http_client_retries: 3,
            http_client_buffer_size: 3,
            timeout_duration_ms: 5000,
        },
        &core.handle(),
    );
    let client_handle = client.handle();
    core.handle().spawn(client.stream().for_each(|_| Ok(())));
    (core, client_handle)
}

pub fn create_user_role(
    user_id: UserId,
    core: &mut tokio_core::reactor::Core,
    http_client: &HttpClientHandle,
    base_url: String,
) -> Result<UserRole, client::Error> {
    let new_role = NewUserRole {
        user_id,
        name: StoresRole::User,
        id: RoleId::new(),
        data: None,
    };

    let super_user_id = UserId(1);

    let body: String = serde_json::to_string(&new_role).unwrap().to_string();
    core.run(http_client.request_with_auth_header::<UserRole>(
        Method::Post,
        format!("{}/{}", base_url, "roles"),
        Some(body),
        Some(super_user_id.to_string()),
    ))
}

pub fn create_user_store_role(
    user_id: UserId,
    store_id: StoreId,
    core: &mut tokio_core::reactor::Core,
    http_client: &HttpClientHandle,
    base_url: String,
) -> Result<UserRole, client::Error> {
    let new_role = NewUserRole {
        user_id,
        name: StoresRole::StoreManager,
        id: RoleId::new(),
        data: Some(serde_json::to_value(store_id.0).unwrap()),
    };

    let super_user_id = UserId(1);

    let body: String = serde_json::to_string(&new_role).unwrap().to_string();
    core.run(http_client.request_with_auth_header::<UserRole>(
        Method::Post,
        format!("{}/{}", base_url, "roles"),
        Some(body),
        Some(super_user_id.to_string()),
    ))
}

pub fn delete_role(
    user_id: UserId,
    core: &mut tokio_core::reactor::Core,
    http_client: &HttpClientHandle,
    url: String,
) -> Result<Vec<UserRole>, client::Error> {
    let super_user_id = UserId(1);
    core.run(http_client.request_with_auth_header::<Vec<UserRole>>(
        Method::Delete,
        format!("{}/roles/by-user-id/{}", url, user_id.to_string()),
        None,
        Some(super_user_id.to_string()),
    ))
}
