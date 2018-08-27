extern crate delivery_lib as lib;
extern crate futures;
extern crate hyper;
extern crate serde_json;
extern crate stq_http;
extern crate stq_static_resources;
extern crate stq_types;
extern crate tokio_core;

pub mod common;

use hyper::Method;

use lib::models::*;
use stq_types::*;

use std::result;
use stq_http::client::{self, ClientHandle as HttpClientHandle};

static MOCK_RESTRICTION_ENDPOINT: &'static str = "restrictions";

fn create_update_restriction(rest: String) -> UpdateRestriction {
    UpdateRestriction {
        name: rest,
        max_weight: Some(1f64),
        max_size: Some(1f64),
    }
}

// super user
fn create_restriction(
    name: String,
    core: &mut tokio_core::reactor::Core,
    http_client: &HttpClientHandle,
    base_url: String,
) -> result::Result<Restriction, client::Error> {
    let new_restriction = NewRestriction {
        name: name,
        max_weight: 0f64,
        max_size: 0f64,
    };

    let user_id = UserId(1);

    let body: String = serde_json::to_string(&new_restriction).unwrap().to_string();
    let create_result = core.run(http_client.request_with_auth_header::<Restriction>(
        Method::Post,
        format!("{}/{}", base_url, MOCK_RESTRICTION_ENDPOINT.to_string()),
        Some(body),
        Some(user_id.to_string()),
    ));

    create_result
}

// super user
fn delete_restriction(
    core: &mut tokio_core::reactor::Core,
    http_client: &HttpClientHandle,
    url: String,
) -> result::Result<Restriction, client::Error> {
    let user_id = UserId(1);
    core.run(http_client.request_with_auth_header::<Restriction>(Method::Delete, url, None, Some(user_id.to_string())))
}

fn get_url_request_by_param(base_url: String, rest: String) -> String {
    format!("{}/{}?name={}", base_url, MOCK_RESTRICTION_ENDPOINT, rest)
}

fn get_url_request(base_url: String) -> String {
    format!("{}/{}", base_url, MOCK_RESTRICTION_ENDPOINT)
}

// test restriction by superuser
#[test]
fn test_restriction_superuser_crud() {
    let (mut core, http_client) = common::make_utils();
    let base_url = common::setup();
    let user_id = UserId(1);
    let restriction_name = "rest_super_user_mock".to_string();
    let url_rud = get_url_request_by_param(base_url.clone(), restriction_name.clone());

    // create
    println!("run create new restriction {}", restriction_name);
    let create_result = create_restriction(restriction_name.clone(), &mut core, &http_client, base_url.clone());
    println!("{:?}", create_result);
    assert!(create_result.is_ok());

    // read
    println!("run read restriction {}", restriction_name);
    let read_result =
        core.run(http_client.request_with_auth_header::<Restriction>(Method::Get, url_rud.clone(), None, Some(user_id.to_string())));
    println!("{:?}", read_result);
    assert!(read_result.is_ok());

    // update
    println!("run update restriction {}", restriction_name);
    let update_restriction = create_update_restriction(restriction_name.clone());
    let update_body: String = serde_json::to_string(&update_restriction).unwrap().to_string();
    let update_result = core.run(http_client.request_with_auth_header::<Restriction>(
        Method::Put,
        get_url_request(base_url.clone()),
        Some(update_body),
        Some(user_id.to_string()),
    ));
    println!("{:?}", update_result);
    assert!(update_result.is_ok());

    // delete
    println!("run delete restriction {}", restriction_name);
    let delete_result = delete_restriction(&mut core, &http_client, url_rud.clone());
    assert!(delete_result.is_ok());
}

// test restriction by regular user
#[test]
fn test_restriction_regular_user_crud() {
    let (mut core, http_client) = common::make_utils();
    let base_url = common::setup();

    // create user for test acl
    let user_id = UserId(1123);
    let create_role_result = common::create_user_role(user_id.clone(), &mut core, &http_client, base_url.clone());
    assert!(create_role_result.is_ok());

    let restriction_name = "rest_regular_user_mock".to_string();
    let url_rud = get_url_request_by_param(base_url.clone(), restriction_name.clone());

    // create by super for test
    println!("run create new restriction {}", restriction_name);
    let create_result = create_restriction(restriction_name.clone(), &mut core, &http_client, base_url.clone());
    println!("{:?}", create_result);
    assert!(create_result.is_ok());

    // read
    println!("run read restriction {}", restriction_name);
    let read_result =
        core.run(http_client.request_with_auth_header::<Restriction>(Method::Get, url_rud.clone(), None, Some(user_id.to_string())));
    println!("{:?}", read_result);
    assert!(read_result.is_ok());

    // update
    println!("run update restriction {}", restriction_name);
    let update_restriction = create_update_restriction(restriction_name.clone());
    let update_body: String = serde_json::to_string(&update_restriction).unwrap().to_string();
    let update_result = core.run(http_client.request_with_auth_header::<Restriction>(
        Method::Put,
        get_url_request(base_url.clone()),
        Some(update_body),
        Some(user_id.to_string()),
    ));
    println!("{:?}", update_result);
    assert!(update_result.is_err());

    // delete by super for test
    println!("run delete restriction {}", restriction_name);
    let delete_result = delete_restriction(&mut core, &http_client, url_rud.clone());
    assert!(delete_result.is_ok());

    // delete user role
    let delete_result = common::delete_role(user_id.clone(), &mut core, &http_client, base_url.clone());
    assert!(delete_result.is_ok());
}

// test update restriction without authorization data
#[test]
fn test_update_restriction_unauthorized() {
    let (mut core, http_client) = common::make_utils();
    let base_url = common::setup();
    let restriction_name = "rest_no_user_mock".to_string();
    let url_rud = get_url_request_by_param(base_url.clone(), restriction_name.clone());

    // create by super for test
    println!("run create new restriction {}", restriction_name);
    let create_result = create_restriction(restriction_name.clone(), &mut core, &http_client, base_url.clone());
    println!("{:?}", create_result);
    assert!(create_result.is_ok());

    // read
    println!("run read restriction {}", restriction_name);
    let read_result = core.run(http_client.request_with_auth_header::<Restriction>(Method::Get, url_rud.clone(), None, None));
    println!("{:?}", read_result);
    assert!(read_result.is_ok());

    // update
    println!("run update restriction {}", restriction_name);
    let update_restriction = create_update_restriction(restriction_name.clone());
    let update_body: String = serde_json::to_string(&update_restriction).unwrap().to_string();
    let update_result = core.run(http_client.request_with_auth_header::<Restriction>(
        Method::Put,
        get_url_request(base_url.clone()),
        Some(update_body),
        None,
    ));
    println!("{:?}", update_result);
    assert!(update_result.is_err());

    // delete by super for test
    println!("run delete restriction {}", restriction_name);
    let delete_result = delete_restriction(&mut core, &http_client, url_rud.clone());
    assert!(delete_result.is_ok());
}
