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
use std::result;
use stq_http::client::{self, ClientHandle as HttpClientHandle};
use stq_types::*;
use tokio_core::reactor::Core;

static MOCK_RESTRICTION_NAME: &'static str = "restriction_mock";
static MOCK_RESTRICTION_ENDPOINT: &'static str = "restrictions";

fn create_new_restriction() -> NewRestriction {
    NewRestriction {
        name: MOCK_RESTRICTION_NAME.to_string(),
        max_weight: 0f64,
        max_size: 0f64,
    }
}

fn create_update_restriction() -> UpdateRestriction {
    UpdateRestriction {
        name: MOCK_RESTRICTION_NAME.to_string(),
        max_weight: 1f64,
        max_size: 1f64,
    }
}

// test restriction by superuser
#[test]
fn test_restriction_superuser_crud() {
    let (mut core, http_client) = common::make_utils();
    let base_url = common::setup();
    let user_id = UserId(1);
    let url_rud = format!("{}/{}/by-name/{}", base_url, MOCK_RESTRICTION_ENDPOINT, MOCK_RESTRICTION_NAME);
    
    // create
    println!("run create new restriction");
    let new_restriction = create_new_restriction();
    let body: String = serde_json::to_string(&new_restriction).unwrap().to_string();
    let create_result = core.run(http_client.request_with_auth_header::<Restriction>(
            Method::Post,
            format!("{}/{}", base_url, MOCK_RESTRICTION_ENDPOINT.to_string()),
            Some(body),
            Some(user_id.to_string()),
        ));
    println!("{:?}", create_result);
    assert!(create_result.is_ok());

    // read
    println!("run read restriction");
    let read_result = core.run(http_client.request_with_auth_header::<Restriction>(
            Method::Get,
            url_rud.clone(),
            None,
            Some(user_id.to_string()),
        ));
    println!("{:?}", read_result);
    assert!(read_result.is_ok());

    // update
    println!("run update restriction");
    let update_restriction = create_update_restriction();
    let update_body: String = serde_json::to_string(&update_restriction).unwrap().to_string();
    let update_result = core.run(http_client.request_with_auth_header::<Restriction>(
            Method::Put,
            url_rud.clone(),
            Some(update_body),
            Some(user_id.to_string()),
        ));
    println!("{:?}", update_result);
    assert!(update_result.is_ok());

    // delete
    println!("run delete restriction");
    let delete_result = core.run(http_client.request_with_auth_header::<Restriction>(
            Method::Get,
            url_rud.clone(),
            None,
            Some(user_id.to_string()),
        ));
    assert!(delete_result.is_ok());
}

/*// test restriction by regular user
#[test]
fn test_restriction_regular_user_crud() {
    let base_url = common::setup();
    let restrictions = init_restrictions_paths();
    let user_id = UserId(123);

    let mut rpc = RpcClient::new(base_url.clone(), Some(user_id));
    for restriction in restrictions.iter() {
        let restriction_result = rpc.request_restriction(Method::Get, restriction.clone(), None);
        assert!(restriction_result.is_err());
    }
}

// test restriction without authorization data
#[test]
fn test_restriction_unauthorized_crud() {
    let base_url = common::setup();
    let restrictions = init_restrictions_paths();

    let mut rpc = RpcClient::new(base_url.clone(), None);
    for restriction in restrictions.iter() {
        let restriction_result = rpc.request_restriction(Method::Get, restriction.clone(), None);
        assert!(restriction_result.is_err());
    }
}*/

/*// test update restriction by superuser
#[test]
fn test_update_restriction_superuser() {
    let base_url = common::setup();
    let restrictions = init_restrictions_paths();
    let user_id = UserId(1);

    let mut rpc = RpcClient::new(base_url.clone(), Some(user_id));
    for restriction in restrictions.iter() {
        let restriction_result = rpc.request_restriction(Method::Put, restriction.clone(), Some(create_restriction_mock()));
        assert!(restriction_result.is_ok());
    }
}

// test update restriction by regular user
#[test]
fn test_update_restriction_regular_user() {
    let base_url = common::setup();
    let restrictions = init_restrictions_paths();
    let user_id = UserId(123);

    let mut rpc = RpcClient::new(base_url.clone(), Some(user_id));
    for restriction in restrictions.iter() {
        let restriction_result = rpc.request_restriction(Method::Put, restriction.clone(), Some(create_restriction_mock()));
        assert!(restriction_result.is_err());
    }
}

// test update restriction without authorization data
#[test]
fn test_update_restriction_unauthorized() {
    let base_url = common::setup();
    let restrictions = init_restrictions_paths();

    let mut rpc = RpcClient::new(base_url.clone(), None);
    for restriction in restrictions.iter() {
        let restriction_result = rpc.request_restriction(Method::Put, restriction.clone(), Some(create_restriction_mock()));
        assert!(restriction_result.is_err());
    }
}
*/
