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

use stq_http::client::{self, ClientHandle as HttpClientHandle};

static MOCK_USER_ADDRESSES_ENDPOINT: &'static str = "users/addresses";

fn create_update_address() -> UpdateUserAddress {
    UpdateUserAddress {
        administrative_area_level_1: None,
        administrative_area_level_2: None,
        country: None,
        locality: Some("dsadada".to_string()),
        political: None,
        postal_code: None,
        route: None,
        street_number: None,
        is_priority: None,
        address: None,
    }
}

fn create_address(
    core: &mut tokio_core::reactor::Core,
    http_client: &HttpClientHandle,
    base_url: String,
    user_id: Option<UserId>,
) -> Result<UserAddress, client::Error> {
    let new_address = NewUserAddress {
        user_id: user_id.unwrap_or(UserId(2)),
        administrative_area_level_1: None,
        administrative_area_level_2: None,
        country: "None".to_string(),
        locality: None,
        political: None,
        postal_code: "None".to_string(),
        route: None,
        street_number: None,
        is_priority: true,
        address: None,
    };

    let body: String = serde_json::to_string(&new_address).unwrap().to_string();
    let create_result = core.run(http_client.request_with_auth_header::<UserAddress>(
        Method::Post,
        get_url_request(base_url),
        Some(body),
        user_id.map(|u| u.to_string()),
    ));

    create_result
}

fn get_url_request_by_user_id(base_url: String, user_id: UserId) -> String {
    format!("{}/users/{}/addresses", base_url, user_id)
}

fn get_url_request_by_address_id(base_url: String, address_id: i32) -> String {
    format!("{}/users/addresses/{}", base_url, address_id)
}

fn get_url_request(base_url: String) -> String {
    format!("{}/{}", base_url, MOCK_USER_ADDRESSES_ENDPOINT)
}

// test address by superuser
#[test]
fn test_address_superuser_crud() {
    let (mut core, http_client) = common::make_utils();
    let base_url = common::setup();
    let user_id = UserId(1);
    // create
    println!("run create new address ");
    let create_result = create_address(&mut core, &http_client, base_url.clone(), Some(user_id));
    println!("{:?}", create_result);
    assert!(create_result.is_ok());

    let address = create_result.unwrap();
    // read search
    println!("run search address by id");
    let read_result = core.run(http_client.request_with_auth_header::<Vec<UserAddress>>(
        Method::Get,
        get_url_request_by_user_id(base_url.clone(), user_id),
        None,
        Some(user_id.to_string()),
    ));
    println!("{:?}", read_result);
    assert!(read_result.is_ok());

    // update
    println!("run update address ");
    let update_address = create_update_address();
    let update_body: String = serde_json::to_string(&update_address).unwrap().to_string();
    let update_result = core.run(http_client.request_with_auth_header::<UserAddress>(
        Method::Put,
        get_url_request_by_address_id(base_url.clone(), address.id),
        Some(update_body),
        Some(user_id.to_string()),
    ));
    println!("{:?}", update_result);
    assert!(update_result.is_ok());

    // delete
    println!("run delete address ");
    let delete_result = core.run(http_client.request_with_auth_header::<UserAddress>(
        Method::Delete,
        get_url_request_by_address_id(base_url.clone(), address.id),
        None,
        Some(user_id.to_string()),
    ));
    assert!(delete_result.is_ok());
}

// test address by regular user
#[test]
fn test_address_regular_user_crud() {
    let (mut core, http_client) = common::make_utils();
    let base_url = common::setup();

    // create user for test acl
    let user_id = UserId(1123);
    let create_role_result = common::create_user_role(user_id.clone(), &mut core, &http_client, base_url.clone());
    assert!(create_role_result.is_ok());

    // create
    println!("run create new address ");
    let create_result = create_address(&mut core, &http_client, base_url.clone(), Some(user_id));
    println!("{:?}", create_result);
    assert!(create_result.is_ok());

    let address = create_result.unwrap();
    // read search
    println!("run search address by id");
    let read_result = core.run(http_client.request_with_auth_header::<Vec<UserAddress>>(
        Method::Get,
        get_url_request_by_user_id(base_url.clone(), user_id),
        None,
        Some(user_id.to_string()),
    ));
    println!("{:?}", read_result);
    assert!(read_result.is_ok());

    // update
    println!("run update address ");
    let update_address = create_update_address();
    let update_body: String = serde_json::to_string(&update_address).unwrap().to_string();
    let update_result = core.run(http_client.request_with_auth_header::<UserAddress>(
        Method::Put,
        get_url_request_by_address_id(base_url.clone(), address.id),
        Some(update_body),
        Some(user_id.to_string()),
    ));
    println!("{:?}", update_result);
    assert!(update_result.is_ok());

    // delete
    println!("run delete address ");
    let delete_result = core.run(http_client.request_with_auth_header::<UserAddress>(
        Method::Delete,
        get_url_request_by_address_id(base_url.clone(), address.id),
        None,
        Some(user_id.to_string()),
    ));
    assert!(delete_result.is_ok());
}

// test update address without authorization data
#[test]
fn test_address_unauthorized() {
    let (mut core, http_client) = common::make_utils();
    let base_url = common::setup();

    // create
    println!("run create new address ");
    let create_result = create_address(&mut core, &http_client, base_url.clone(), None);
    println!("{:?}", create_result);
    assert!(create_result.is_err());

    // create by super user
    println!("run create new address by super user");
    let create_result = create_address(&mut core, &http_client, base_url.clone(), Some(UserId(1)));
    println!("{:?}", create_result);
    assert!(create_result.is_ok());

    let address = create_result.unwrap();
    // read search
    println!("run search address by id");
    let read_result = core.run(http_client.request_with_auth_header::<Vec<UserAddress>>(
        Method::Get,
        get_url_request_by_user_id(base_url.clone(), UserId(1)),
        None,
        None,
    ));
    println!("{:?}", read_result);
    assert!(read_result.is_err());

    // update
    println!("run update address ");
    let update_address = create_update_address();
    let update_body: String = serde_json::to_string(&update_address).unwrap().to_string();
    let update_result = core.run(http_client.request_with_auth_header::<UserAddress>(
        Method::Put,
        get_url_request_by_address_id(base_url.clone(), address.id),
        Some(update_body),
        None,
    ));
    println!("{:?}", update_result);
    assert!(update_result.is_err());

    // delete
    println!("run delete address ");
    let delete_result = core.run(http_client.request_with_auth_header::<UserAddress>(
        Method::Delete,
        get_url_request_by_address_id(base_url.clone(), address.id),
        None,
        None,
    ));
    println!("{:?}", delete_result);
    assert!(delete_result.is_err());

    // delete by super user
    println!("run delete address by super user ");
    let delete_result = core.run(http_client.request_with_auth_header::<UserAddress>(
        Method::Delete,
        get_url_request_by_address_id(base_url.clone(), address.id),
        None,
        Some("1".to_string()),
    ));
    println!("{:?}", delete_result);
    assert!(delete_result.is_ok());
}
