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

use stq_http::client::{self, ClientHandle as HttpClientHandle};
use stq_static_resources::DeliveryCompany;
use stq_types::*;

use lib::models::*;

static MOCK_SHIPPING_INTERNATIONAL_ENDPOINT: &'static str = "shipping/international";

fn create_update_international_shipping(price: ProductPrice) -> UpdateInternationalShipping {
    UpdateInternationalShipping {
        companies: Some(vec![InternationalShippingCompany {
            company: DeliveryCompany::DHL,
            price: Some(price),
            duration_days: None,
            countries: vec!["all".to_string()],
        }]),
    }
}

// super user
fn create_international_shipping(
    base_product_id: BaseProductId,
    core: &mut tokio_core::reactor::Core,
    http_client: &HttpClientHandle,
    base_url: String,
    user_id: Option<String>,
) -> Result<InternationalShipping, client::Error> {
    let new_international_shipping = NewInternationalShipping {
        base_product_id,
        store_id: StoreId(1),
        companies: vec![InternationalShippingCompany {
            company: DeliveryCompany::DHL,
            price: None,
            duration_days: None,
            countries: vec![],
        }],
    };

    let body: String = serde_json::to_string(&new_international_shipping).unwrap().to_string();
    let create_result = core.run(http_client.request_with_auth_header::<InternationalShipping>(
        Method::Post,
        format!("{}/{}", base_url, MOCK_SHIPPING_INTERNATIONAL_ENDPOINT.to_string()),
        Some(body),
        user_id,
    ));

    create_result
}

// super user
fn delete_international_shipping(
    core: &mut tokio_core::reactor::Core,
    http_client: &HttpClientHandle,
    url: String,
) -> Result<InternationalShipping, client::Error> {
    let user_id = UserId(1);
    core.run(http_client.request_with_auth_header::<InternationalShipping>(Method::Delete, url, None, Some(user_id.to_string())))
}

fn get_url_request_by_base_product_id(base_url: String, base_product_id: BaseProductId) -> String {
    format!("{}/{}/{}", base_url, MOCK_SHIPPING_INTERNATIONAL_ENDPOINT, base_product_id)
}

// test international_shipping by superuser
#[test]
fn test_international_shipping_superuser_crud() {
    let (mut core, http_client) = common::make_utils();
    let base_url = common::setup();
    let user_id = UserId(1);
    let base_product_id = BaseProductId(1);
    let price = ProductPrice(100f64);
    let url_crud = get_url_request_by_base_product_id(base_url.clone(), base_product_id);

    // create
    println!("run create new international_shipping for base_product {}", base_product_id);
    let create_result = create_international_shipping(
        base_product_id,
        &mut core,
        &http_client,
        base_url.clone(),
        Some(user_id.to_string()),
    );
    println!("{:?}", create_result);
    assert!(create_result.is_ok());

    // read
    println!("run read international_shipping for base_product {}", base_product_id);
    let read_result = core.run(http_client.request_with_auth_header::<InternationalShipping>(
        Method::Get,
        url_crud.clone(),
        None,
        Some(user_id.to_string()),
    ));
    println!("{:?}", read_result);
    assert!(read_result.is_ok());

    // update
    println!("run update international_shipping for base_product {}", base_product_id);
    let update_international_shipping = create_update_international_shipping(price);
    let update_body: String = serde_json::to_string(&update_international_shipping).unwrap().to_string();
    let update_result = core.run(http_client.request_with_auth_header::<InternationalShipping>(
        Method::Put,
        url_crud.clone(),
        Some(update_body),
        Some(user_id.to_string()),
    ));
    println!("{:?}", update_result);
    assert!(update_result.is_ok());

    // delete
    println!("run delete international_shipping for base_product {}", base_product_id);
    let delete_result = delete_international_shipping(&mut core, &http_client, url_crud.clone());
    assert!(delete_result.is_ok());
}

// test international_shipping by regular user
#[test]
fn test_international_shipping_regular_user_crud() {
    let (mut core, http_client) = common::make_utils();
    let base_url = common::setup();
    let base_product_id = BaseProductId(2);
    let price = ProductPrice(100f64);
    let url_crud = get_url_request_by_base_product_id(base_url.clone(), base_product_id);

    // create user for test acl
    let user_id = UserId(2);
    let create_role_result = common::create_user_role(user_id.clone(), &mut core, &http_client, base_url.clone());
    assert!(create_role_result.is_ok());

    // create
    println!(
        "run create new international_shipping for base_product {} for regular user",
        base_product_id
    );
    let create_result = create_international_shipping(
        base_product_id,
        &mut core,
        &http_client,
        base_url.clone(),
        Some(user_id.to_string()),
    );
    println!("{:?}", create_result);
    assert!(create_result.is_err());

    // create by super user
    println!("run create new international_shipping for base_product {}", base_product_id);
    let create_result = create_international_shipping(
        base_product_id,
        &mut core,
        &http_client,
        base_url.clone(),
        Some(UserId(1).to_string()),
    );
    println!("{:?}", create_result);
    assert!(create_result.is_err());

    // read
    println!("run read international_shipping for base_product {}", base_product_id);
    let read_result = core.run(http_client.request_with_auth_header::<InternationalShipping>(
        Method::Get,
        url_crud.clone(),
        None,
        Some(user_id.to_string()),
    ));
    println!("{:?}", read_result);
    assert!(read_result.is_ok());

    // update
    println!("run update international_shipping for base_product {}", base_product_id);
    let update_international_shipping = create_update_international_shipping(price);
    let update_body: String = serde_json::to_string(&update_international_shipping).unwrap().to_string();
    let update_result = core.run(http_client.request_with_auth_header::<InternationalShipping>(
        Method::Put,
        url_crud.clone(),
        Some(update_body),
        Some(user_id.to_string()),
    ));
    println!("{:?}", update_result);
    assert!(update_result.is_err());

    // delete by super for test
    println!("run delete international_shipping for base_product {}", base_product_id);
    let delete_result = delete_international_shipping(&mut core, &http_client, url_crud.clone());
    assert!(delete_result.is_ok());

    // delete user role
    let delete_result = common::delete_role(user_id.clone(), &mut core, &http_client, base_url.clone());
    assert!(delete_result.is_ok());
}

// test update international_shipping without authorization data
#[test]
fn test_international_shipping_unauthorized() {
    let (mut core, http_client) = common::make_utils();
    let base_url = common::setup();
    let base_product_id = BaseProductId(3);
    let price = ProductPrice(100f64);
    let url_crud = get_url_request_by_base_product_id(base_url.clone(), base_product_id);

    // create
    println!("run create new international_shipping for base_product {}", base_product_id);
    let create_result = create_international_shipping(base_product_id, &mut core, &http_client, base_url.clone(), None);
    println!("{:?}", create_result);
    assert!(create_result.is_err());

    // create by super user
    println!("run create new international_shipping for base_product {}", base_product_id);
    let create_result = create_international_shipping(
        base_product_id,
        &mut core,
        &http_client,
        base_url.clone(),
        Some(UserId(1).to_string()),
    );
    println!("{:?}", create_result);
    assert!(create_result.is_err());

    // read
    println!("run read international_shipping for base_product {}", base_product_id);
    let read_result = core.run(http_client.request_with_auth_header::<InternationalShipping>(Method::Get, url_crud.clone(), None, None));
    println!("{:?}", read_result);
    assert!(read_result.is_err());

    // update
    println!("run update international_shipping for base_product {}", base_product_id);
    let update_international_shipping = create_update_international_shipping(price);
    let update_body: String = serde_json::to_string(&update_international_shipping).unwrap().to_string();
    let update_result =
        core.run(http_client.request_with_auth_header::<InternationalShipping>(Method::Put, url_crud.clone(), Some(update_body), None));
    println!("{:?}", update_result);
    assert!(update_result.is_err());

    // delete by super for test
    println!("run delete international_shipping for base_product {}", base_product_id);
    let delete_result = delete_international_shipping(&mut core, &http_client, url_crud.clone());
    assert!(delete_result.is_ok());
}

// test international_shipping by store manager
#[test]
fn test_international_shipping_store_manager() {
    let (mut core, http_client) = common::make_utils();
    let base_url = common::setup();
    let base_product_id = BaseProductId(4);
    let store_id = StoreId(1);
    let price = ProductPrice(100f64);
    let url_crud = get_url_request_by_base_product_id(base_url.clone(), base_product_id);

    // create store_manager for test acl
    let user_id = UserId(3);
    let create_role_result = common::create_user_store_role(user_id.clone(), store_id, &mut core, &http_client, base_url.clone());
    assert!(create_role_result.is_ok());

    // create
    println!("run create new international_shipping for base_product {}", base_product_id);
    let create_result = create_international_shipping(
        base_product_id,
        &mut core,
        &http_client,
        base_url.clone(),
        Some(user_id.to_string()),
    );
    println!("{:?}", create_result);
    assert!(create_result.is_ok());

    // read
    println!("run read international_shipping for base_product {}", base_product_id);
    let read_result = core.run(http_client.request_with_auth_header::<InternationalShipping>(
        Method::Get,
        url_crud.clone(),
        None,
        Some(user_id.to_string()),
    ));
    println!("{:?}", read_result);
    assert!(read_result.is_ok());

    // update
    println!("run update international_shipping for base_product {}", base_product_id);
    let update_international_shipping = create_update_international_shipping(price);
    let update_body: String = serde_json::to_string(&update_international_shipping).unwrap().to_string();
    let update_result = core.run(http_client.request_with_auth_header::<InternationalShipping>(
        Method::Put,
        url_crud.clone(),
        Some(update_body),
        Some(user_id.to_string()),
    ));
    println!("{:?}", update_result);
    assert!(update_result.is_ok());

    // delete by super for test
    println!("run delete international_shipping for base_product {}", base_product_id);
    let delete_result = delete_international_shipping(&mut core, &http_client, url_crud.clone());
    assert!(delete_result.is_ok());

    // delete user role
    let delete_result = common::delete_role(user_id.clone(), &mut core, &http_client, base_url.clone());
    assert!(delete_result.is_ok());
}
