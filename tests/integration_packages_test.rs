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

static MOCK_PACKAGES_ENDPOINT: &'static str = "packages";

fn create_update_package(name: &str) -> UpdatePackages {
    UpdatePackages {
        name: Some(name.to_string()),
        max_size: Some(0),
        min_size: Some(0),
        max_weight: Some(0),
        min_weight: Some(0),
        deliveries_to: Some(vec![]),
    }
}

fn create_package(
    name: String,
    core: &mut tokio_core::reactor::Core,
    http_client: &HttpClientHandle,
    base_url: String,
    user_id: Option<UserId>,
) -> Result<Packages, client::Error> {
    let new = NewPackages {
        name,
        max_size: 0,
        min_size: 0,
        max_weight: 0,
        min_weight: 0,
        deliveries_to: vec![],
    };

    let body: String = serde_json::to_string(&new).unwrap().to_string();
    let create_result = core.run(http_client.request_with_auth_header::<Packages>(
        Method::Post,
        get_url_request(base_url),
        Some(body),
        user_id.map(|u| u.to_string()),
    ));

    create_result
}

fn get_url_request_by_id(base_url: String, package_id: PackageId) -> String {
    format!("{}/{}/{}", base_url, MOCK_PACKAGES_ENDPOINT, package_id)
}

fn get_url_request(base_url: String) -> String {
    format!("{}/{}", base_url, MOCK_PACKAGES_ENDPOINT)
}

#[test]
fn test_package() {
    let (mut core, http_client) = common::make_utils();
    let base_url = common::setup();

    test_package_superuser_crud(&mut core, &http_client, base_url.clone());
    test_package_regular_user_crud(&mut core, &http_client, base_url.clone());
    test_package_unauthorized(&mut core, &http_client, base_url.clone());
}

#[test]
fn test_available_packages_for_user() {
    let (mut core, http_client) = common::make_utils();
    let base_url = common::setup();
    available_packages_for_user(&mut core, &http_client, base_url.clone());
}

// test package by superuser
fn test_package_superuser_crud(core: &mut tokio_core::reactor::Core, http_client: &HttpClientHandle, base_url: String) {
    let user_id = UserId(1);
    let name = "Avia".to_string();
    // create
    println!("run create new package ");
    let create_result = create_package(name.clone(), core, http_client, base_url.clone(), Some(user_id));
    println!("{:?}", create_result);
    assert!(create_result.is_ok());

    let package = create_result.unwrap();
    // read search
    println!("run search package by id");
    let read_result = core.run(http_client.request_with_auth_header::<Packages>(
        Method::Get,
        get_url_request_by_id(base_url.clone(), package.id),
        None,
        Some(user_id.to_string()),
    ));
    println!("{:?}", read_result);
    assert!(read_result.is_ok());

    // update
    println!("run update package ");
    let update_package = create_update_package("UPS USA 2");
    let update_body: String = serde_json::to_string(&update_package).unwrap().to_string();
    let update_result = core.run(http_client.request_with_auth_header::<Packages>(
        Method::Put,
        get_url_request_by_id(base_url.clone(), package.id),
        Some(update_body),
        Some(user_id.to_string()),
    ));
    println!("{:?}", update_result);
    assert!(update_result.is_ok());

    // delete
    println!("run delete package ");
    let delete_result = core.run(http_client.request_with_auth_header::<Packages>(
        Method::Delete,
        get_url_request_by_id(base_url.clone(), package.id),
        None,
        Some(user_id.to_string()),
    ));
    assert!(delete_result.is_ok());
}

// test package by regular user
fn test_package_regular_user_crud(core: &mut tokio_core::reactor::Core, http_client: &HttpClientHandle, base_url: String) {
    // create user for test acl
    let user_id = UserId(1123);
    let create_role_result = common::create_user_role(user_id.clone(), core, http_client, base_url.clone());
    assert!(create_role_result.is_ok());

    let name = "Avia".to_string();
    // create
    println!("run create new package ");
    let create_result = create_package(name.clone(), core, http_client, base_url.clone(), Some(user_id));
    println!("{:?}", create_result);
    assert!(create_result.is_err());

    // create by super user
    println!("run create new package by super user");
    let create_result = create_package(name.clone(), core, http_client, base_url.clone(), Some(UserId(1)));
    println!("{:?}", create_result);
    assert!(create_result.is_ok());

    let package = create_result.unwrap();
    // read search
    println!("run search package by id");
    let read_result = core.run(http_client.request_with_auth_header::<Packages>(
        Method::Get,
        get_url_request_by_id(base_url.clone(), package.id),
        None,
        Some(user_id.to_string()),
    ));
    println!("{:?}", read_result);
    assert!(read_result.is_ok());

    // update
    println!("run update package ");
    let update_package = create_update_package("UPS USA 2");
    let update_body: String = serde_json::to_string(&update_package).unwrap().to_string();
    let update_result = core.run(http_client.request_with_auth_header::<Packages>(
        Method::Put,
        get_url_request_by_id(base_url.clone(), package.id),
        Some(update_body),
        Some(user_id.to_string()),
    ));
    println!("{:?}", update_result);
    assert!(update_result.is_err());

    // delete
    println!("run delete package ");
    let delete_result = core.run(http_client.request_with_auth_header::<Packages>(
        Method::Delete,
        get_url_request_by_id(base_url.clone(), package.id),
        None,
        Some(user_id.to_string()),
    ));
    assert!(delete_result.is_err());

    // delete by super user
    println!("run delete package by super user ");
    let delete_result = core.run(http_client.request_with_auth_header::<Packages>(
        Method::Delete,
        get_url_request_by_id(base_url.clone(), package.id),
        None,
        Some("1".to_string()),
    ));
    assert!(delete_result.is_ok());
}

// test update package without authorization data
fn test_package_unauthorized(core: &mut tokio_core::reactor::Core, http_client: &HttpClientHandle, base_url: String) {
    let name = "Avia".to_string();

    // create
    println!("run create new package ");
    let create_result = create_package(name.clone(), core, http_client, base_url.clone(), None);
    println!("{:?}", create_result);
    assert!(create_result.is_err());

    // create by super user
    println!("run create new package by super user");
    let create_result = create_package(name.clone(), core, http_client, base_url.clone(), Some(UserId(1)));
    println!("{:?}", create_result);
    assert!(create_result.is_ok());

    let package = create_result.unwrap();
    // read search
    println!("run search package by id");
    let read_result = core.run(http_client.request_with_auth_header::<Packages>(
        Method::Get,
        get_url_request_by_id(base_url.clone(), package.id),
        None,
        None,
    ));
    println!("{:?}", read_result);
    assert!(read_result.is_ok());

    // update
    println!("run update package ");
    let update_package = create_update_package("UPS USA 2");
    let update_body: String = serde_json::to_string(&update_package).unwrap().to_string();
    let update_result = core.run(http_client.request_with_auth_header::<Packages>(
        Method::Put,
        get_url_request_by_id(base_url.clone(), package.id),
        Some(update_body),
        None,
    ));
    println!("{:?}", update_result);
    assert!(update_result.is_err());

    // delete
    println!("run delete package ");
    let delete_result = core.run(http_client.request_with_auth_header::<Packages>(
        Method::Delete,
        get_url_request_by_id(base_url.clone(), package.id),
        None,
        None,
    ));
    assert!(delete_result.is_err());

    // delete by super user
    println!("run delete package by super user ");
    let delete_result = core.run(http_client.request_with_auth_header::<Packages>(
        Method::Delete,
        get_url_request_by_id(base_url.clone(), package.id),
        None,
        Some("1".to_string()),
    ));
    assert!(delete_result.is_ok());
}

fn available_packages_for_user(core: &mut tokio_core::reactor::Core, http_client: &HttpClientHandle, base_url: String) {
    let base_product_id = BaseProductId(4);
    let super_user_id = UserId(1);
    let user_country = Alpha3("RUS".to_string());
    let available_packages_for_user_url = format!(
        "{}/available_packages_for_user/{}?user_country={}",
        base_url, base_product_id, user_country
    );
    let shipping = core
        .run(http_client.request_with_auth_header::<AvailableShippingForUser>(
            Method::Get,
            available_packages_for_user_url,
            None,
            Some(super_user_id.to_string()),
        ))
        .unwrap();
    assert!(shipping.packages.is_empty());
    assert!(shipping.pickups.is_none());
}
