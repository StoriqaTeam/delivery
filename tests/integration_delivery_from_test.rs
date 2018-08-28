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
use stq_static_resources::DeliveryCompany;

static MOCK_DELIVERY_FROM_ENDPOINT: &'static str = "delivery_from";

fn create_update_delivery_from(company_id: DeliveryCompany, country: String) -> UpdateDeliveryFrom {
    let restriction_name = format!("{}_{}", company_id, country);
    UpdateDeliveryFrom {
        company_id,
        country,
        restriction_name,
    }
}

// super user
fn create_delivery_from(
    company_id: DeliveryCompany,
    country: String,
    core: &mut tokio_core::reactor::Core,
    http_client: &HttpClientHandle,
    base_url: String,
) -> result::Result<DeliveryFrom, client::Error> {
    let restriction_name = format!("{}_{}", company_id, country);
    let new_delivery_from = NewDeliveryFrom {
        company_id,
        country,
        restriction_name,
    };

    let user_id = UserId(1);

    let body: String = serde_json::to_string(&new_delivery_from).unwrap().to_string();
    let create_result = core.run(http_client.request_with_auth_header::<DeliveryFrom>(
        Method::Post,
        format!("{}/{}", base_url, MOCK_DELIVERY_FROM_ENDPOINT.to_string()),
        Some(body),
        Some(user_id.to_string()),
    ));

    create_result
}

// super user
fn delete_delivery_from(
    company_id: DeliveryCompany,
    country: String,
    core: &mut tokio_core::reactor::Core,
    http_client: &HttpClientHandle,
    url: String,
) -> result::Result<DeliveryFrom, client::Error> {
    let user_id = UserId(1);

    let url = format!("{}?company_id={}&country={}", url, company_id, country);
    core.run(http_client.request_with_auth_header::<DeliveryFrom>(Method::Delete, url, None, Some(user_id.to_string())))
}

fn get_url_request_by_filter_company(base_url: String, company_id: DeliveryCompany) -> String {
    format!(
        "{}/{}/search/filters/company?company_id={}",
        base_url, MOCK_DELIVERY_FROM_ENDPOINT, company_id
    )
}

fn get_url_request(base_url: String) -> String {
    format!("{}/{}", base_url, MOCK_DELIVERY_FROM_ENDPOINT)
}

// test delivery_from by superuser
#[test]
fn test_delivery_from_superuser_crud() {
    let (mut core, http_client) = common::make_utils();
    let base_url = common::setup();
    let user_id = UserId(1);
    let country = "US".to_string();
    // create
    println!("run create new delivery_from ");
    let create_result = create_delivery_from(DeliveryCompany::DHL, country.clone(), &mut core, &http_client, base_url.clone());
    println!("{:?}", create_result);
    assert!(create_result.is_ok());

    // read search
    println!("run search delivery_from by company");
    let read_result1 = core.run(http_client.request_with_auth_header::<Vec<DeliveryFrom>>(
        Method::Get,
        get_url_request_by_filter_company(base_url.clone(), DeliveryCompany::DHL),
        None,
        Some(user_id.to_string()),
    ));
    println!("{:?}", read_result1);
    assert!(read_result1.is_ok());

    // update
    println!("run update delivery_from ");
    let update_delivery_from = create_update_delivery_from(DeliveryCompany::DHL, country.clone());
    let update_body: String = serde_json::to_string(&update_delivery_from).unwrap().to_string();
    let update_result = core.run(http_client.request_with_auth_header::<DeliveryFrom>(
        Method::Put,
        get_url_request(base_url.clone()),
        Some(update_body),
        Some(user_id.to_string()),
    ));
    println!("{:?}", update_result);
    assert!(update_result.is_ok());

    // delete
    println!("run delete delivery_from ");

    let delete_result = delete_delivery_from(
        DeliveryCompany::DHL,
        country.clone(),
        &mut core,
        &http_client,
        get_url_request(base_url.clone()),
    );
    assert!(delete_result.is_ok());
}

// test delivery_from by regular user
#[test]
fn test_delivery_from_regular_user_crud() {
    let (mut core, http_client) = common::make_utils();
    let base_url = common::setup();
    let country = "RU".to_string();

    // create user for test acl
    let user_id = UserId(1123);
    let create_role_result = common::create_user_role(user_id.clone(), &mut core, &http_client, base_url.clone());
    assert!(create_role_result.is_ok());

    // create by super for test
    println!("run create new delivery_from ");
    let create_result = create_delivery_from(DeliveryCompany::DHL, country.clone(), &mut core, &http_client, base_url.clone());
    println!("{:?}", create_result);
    assert!(create_result.is_ok());

    // read search
    println!("run search delivery_from by company");
    let read_result1 = core.run(http_client.request_with_auth_header::<Vec<DeliveryFrom>>(
        Method::Get,
        get_url_request_by_filter_company(base_url.clone(), DeliveryCompany::DHL),
        None,
        Some(user_id.to_string()),
    ));
    println!("{:?}", read_result1);
    assert!(read_result1.is_ok());

    // update
    println!("run update delivery_from ");
    let update_delivery_from = create_update_delivery_from(DeliveryCompany::DHL, country.clone());
    let update_body: String = serde_json::to_string(&update_delivery_from).unwrap().to_string();
    let update_result = core.run(http_client.request_with_auth_header::<DeliveryFrom>(
        Method::Put,
        get_url_request(base_url.clone()),
        Some(update_body),
        Some(user_id.to_string()),
    ));
    println!("{:?}", update_result);
    assert!(update_result.is_err());

    // delete by super for test
    println!("run delete delivery_from ");

    let delete_result = delete_delivery_from(
        DeliveryCompany::DHL,
        country.clone(),
        &mut core,
        &http_client,
        get_url_request(base_url.clone()),
    );
    assert!(delete_result.is_ok());

    // delete user role
    let delete_result = common::delete_role(user_id.clone(), &mut core, &http_client, base_url.clone());
    assert!(delete_result.is_ok());
}

// test update delivery_from without authorization data
#[test]
fn test_update_delivery_from_unauthorized() {
    let (mut core, http_client) = common::make_utils();
    let base_url = common::setup();
    let country = "UK".to_string();

    // create by super for test
    println!("run create new delivery_from ");
    let create_result = create_delivery_from(DeliveryCompany::DHL, "UK".to_string(), &mut core, &http_client, base_url.clone());
    println!("{:?}", create_result);
    assert!(create_result.is_ok());

    // read search
    println!("run search delivery_from by company");
    let read_result1 = core.run(http_client.request_with_auth_header::<Vec<DeliveryFrom>>(
        Method::Get,
        get_url_request_by_filter_company(base_url.clone(), DeliveryCompany::DHL),
        None,
        None,
    ));
    println!("{:?}", read_result1);
    assert!(read_result1.is_ok());

    // update
    println!("run update delivery_from ");
    let update_delivery_from = create_update_delivery_from(DeliveryCompany::DHL, country.clone());
    let update_body: String = serde_json::to_string(&update_delivery_from).unwrap().to_string();
    let update_result = core.run(http_client.request_with_auth_header::<DeliveryFrom>(
        Method::Put,
        get_url_request(base_url.clone()),
        Some(update_body),
        None,
    ));
    println!("{:?}", update_result);
    assert!(update_result.is_err());

    // delete by super for test
    println!("run delete delivery_from ");
    let delete_result = delete_delivery_from(
        DeliveryCompany::DHL,
        country.clone(),
        &mut core,
        &http_client,
        get_url_request(base_url.clone()),
    );
    assert!(delete_result.is_ok());
}