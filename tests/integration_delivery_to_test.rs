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

static MOCK_DELIVERY_TO_ENDPOINT: &'static str = "delivery_to";

fn create_update_delivery_to(company_id: DeliveryCompany, country: String) -> UpdateDeliveryTo {
    UpdateDeliveryTo {
        company_id,
        country,
        additional_info: None,
    }
}

// super user
fn create_delivery_to(
    company_id: DeliveryCompany,
    country: String,
    core: &mut tokio_core::reactor::Core,
    http_client: &HttpClientHandle,
    base_url: String,
) -> result::Result<DeliveryTo, client::Error> {
    let new_delivery_to = NewDeliveryTo {
        company_id,
        country,
        additional_info: None,
    };

    let user_id = UserId(1);

    let body: String = serde_json::to_string(&new_delivery_to).unwrap().to_string();
    let create_result = core.run(http_client.request_with_auth_header::<DeliveryTo>(
        Method::Post,
        format!("{}/{}", base_url, MOCK_DELIVERY_TO_ENDPOINT.to_string()),
        Some(body),
        Some(user_id.to_string()),
    ));

    create_result
}

fn create_user_role(
    user_id: UserId,
    core: &mut tokio_core::reactor::Core,
    http_client: &HttpClientHandle,
    base_url: String,
) -> result::Result<UserRole, client::Error> {
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

fn delete_role(
    user_id: UserId,
    core: &mut tokio_core::reactor::Core,
    http_client: &HttpClientHandle,
    url: String,
) -> result::Result<Vec<UserRole>, client::Error> {
    let super_user_id = UserId(1);
    core.run(http_client.request_with_auth_header::<Vec<UserRole>>(
        Method::Delete,
        format!("{}/roles/by-user-id/{}", url, user_id.to_string()),
        None,
        Some(super_user_id.to_string()),
    ))
}

// super user
fn delete_delivery_to(
    company_id: DeliveryCompany,
    country: String,
    core: &mut tokio_core::reactor::Core,
    http_client: &HttpClientHandle,
    url: String,
) -> result::Result<DeliveryTo, client::Error> {
    let user_id = UserId(1);

    let url = format!("{}?company_id={}&country={}", url, company_id, country);
    core.run(http_client.request_with_auth_header::<DeliveryTo>(Method::Delete, url, None, Some(user_id.to_string())))
}

fn get_url_request_by_filter_company(base_url: String, company_id: DeliveryCompany) -> String {
    format!(
        "{}/{}/search/filters/company?company_id={}",
        base_url, MOCK_DELIVERY_TO_ENDPOINT, company_id
    )
}

fn get_url_request_by_filter_country(base_url: String, country: String) -> String {
    format!(
        "{}/{}/search/filters/country?country={}",
        base_url, MOCK_DELIVERY_TO_ENDPOINT, country
    )
}

fn get_url_request(base_url: String) -> String {
    format!("{}/{}", base_url, MOCK_DELIVERY_TO_ENDPOINT)
}

// test delivery_to by superuser
#[test]
fn test_delivery_to_superuser_crud() {
    let (mut core, http_client) = common::make_utils();
    let base_url = common::setup();
    let user_id = UserId(1);
    let country = "US".to_string();
    // create
    println!("run create new delivery_to ");
    let create_result = create_delivery_to(DeliveryCompany::DHL, country.clone(), &mut core, &http_client, base_url.clone());
    println!("{:?}", create_result);
    assert!(create_result.is_ok());

    // read search
    println!("run search delivery_to by company");
    let read_result1 = core.run(http_client.request_with_auth_header::<Vec<DeliveryTo>>(
        Method::Get,
        get_url_request_by_filter_company(base_url.clone(), DeliveryCompany::DHL),
        None,
        Some(user_id.to_string()),
    ));
    println!("{:?}", read_result1);
    assert!(read_result1.is_ok());

    println!("run search delivery_to by country");
    let read_result2 = core.run(http_client.request_with_auth_header::<Vec<DeliveryTo>>(
        Method::Get,
        get_url_request_by_filter_country(base_url.clone(), country.clone()),
        None,
        Some(user_id.to_string()),
    ));
    println!("{:?}", read_result2);
    assert!(read_result2.is_ok());

    // update
    println!("run update delivery_to ");
    let update_delivery_to = create_update_delivery_to(DeliveryCompany::DHL, country.clone());
    let update_body: String = serde_json::to_string(&update_delivery_to).unwrap().to_string();
    let update_result = core.run(http_client.request_with_auth_header::<DeliveryTo>(
        Method::Put,
        get_url_request(base_url.clone()),
        Some(update_body),
        Some(user_id.to_string()),
    ));
    println!("{:?}", update_result);
    assert!(update_result.is_ok());

    // delete
    println!("run delete delivery_to ");

    let delete_result = delete_delivery_to(
        DeliveryCompany::DHL,
        country.clone(),
        &mut core,
        &http_client,
        get_url_request(base_url.clone()),
    );
    assert!(delete_result.is_ok());
}

// test delivery_to by regular user
#[test]
fn test_delivery_to_regular_user_crud() {
    let (mut core, http_client) = common::make_utils();
    let base_url = common::setup();
    let country = "RU".to_string();

    // create user for test acl
    let user_id = UserId(1123);
    let create_role_result = create_user_role(user_id.clone(), &mut core, &http_client, base_url.clone());
    assert!(create_role_result.is_ok());

    // create by super for test
    println!("run create new delivery_to ");
    let create_result = create_delivery_to(DeliveryCompany::DHL, country.clone(), &mut core, &http_client, base_url.clone());
    println!("{:?}", create_result);
    assert!(create_result.is_ok());

    // read search
    println!("run search delivery_to by company");
    let read_result1 = core.run(http_client.request_with_auth_header::<Vec<DeliveryTo>>(
        Method::Get,
        get_url_request_by_filter_company(base_url.clone(), DeliveryCompany::DHL),
        None,
        Some(user_id.to_string()),
    ));
    println!("{:?}", read_result1);
    assert!(read_result1.is_ok());

    println!("run search delivery_to by country");
    let read_result2 = core.run(http_client.request_with_auth_header::<Vec<DeliveryTo>>(
        Method::Get,
        get_url_request_by_filter_country(base_url.clone(), country.clone()),
        None,
        Some(user_id.to_string()),
    ));
    println!("{:?}", read_result2);
    assert!(read_result2.is_ok());

    // update
    println!("run update delivery_to ");
    let update_delivery_to = create_update_delivery_to(DeliveryCompany::DHL, country.clone());
    let update_body: String = serde_json::to_string(&update_delivery_to).unwrap().to_string();
    let update_result = core.run(http_client.request_with_auth_header::<DeliveryTo>(
        Method::Put,
        get_url_request(base_url.clone()),
        Some(update_body),
        Some(user_id.to_string()),
    ));
    println!("{:?}", update_result);
    assert!(update_result.is_err());

    // delete by super for test
    println!("run delete delivery_to ");

    let delete_result = delete_delivery_to(
        DeliveryCompany::DHL,
        country.clone(),
        &mut core,
        &http_client,
        get_url_request(base_url.clone()),
    );
    assert!(delete_result.is_ok());

    // delete user role
    let delete_result = delete_role(user_id.clone(), &mut core, &http_client, base_url.clone());
    assert!(delete_result.is_ok());
}

// test update delivery_to without authorization data
#[test]
fn test_update_delivery_to_unauthorized() {
    let (mut core, http_client) = common::make_utils();
    let base_url = common::setup();
    let country = "UK".to_string();

    // create by super for test
    println!("run create new delivery_to ");
    let create_result = create_delivery_to(DeliveryCompany::DHL, "UK".to_string(), &mut core, &http_client, base_url.clone());
    println!("{:?}", create_result);
    assert!(create_result.is_ok());

    // read search
    println!("run search delivery_to by company");
    let read_result1 = core.run(http_client.request_with_auth_header::<Vec<DeliveryTo>>(
        Method::Get,
        get_url_request_by_filter_company(base_url.clone(), DeliveryCompany::DHL),
        None,
        None,
    ));
    println!("{:?}", read_result1);
    assert!(read_result1.is_ok());

    println!("run search delivery_to by country");
    let read_result2 = core.run(http_client.request_with_auth_header::<Vec<DeliveryTo>>(
        Method::Get,
        get_url_request_by_filter_country(base_url.clone(), country.clone()),
        None,
        None,
    ));
    println!("{:?}", read_result2);
    assert!(read_result2.is_ok());

    // update
    println!("run update delivery_to ");
    let update_delivery_to = create_update_delivery_to(DeliveryCompany::DHL, country.clone());
    let update_body: String = serde_json::to_string(&update_delivery_to).unwrap().to_string();
    let update_result = core.run(http_client.request_with_auth_header::<DeliveryTo>(
        Method::Put,
        get_url_request(base_url.clone()),
        Some(update_body),
        None,
    ));
    println!("{:?}", update_result);
    assert!(update_result.is_err());

    // delete by super for test
    println!("run delete delivery_to ");
    let delete_result = delete_delivery_to(
        DeliveryCompany::DHL,
        country.clone(),
        &mut core,
        &http_client,
        get_url_request(base_url.clone()),
    );
    assert!(delete_result.is_ok());
}
