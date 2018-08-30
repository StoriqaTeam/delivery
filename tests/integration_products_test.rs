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
use stq_types::*;

use lib::models::*;

static MOCK_PRODUCTS_ENDPOINT: &'static str = "products";

fn create_update_products(price: f64) -> UpdateProducts {
    UpdateProducts {
        price: Some(ProductPrice(price)),
        deliveries_to: None,
    }
}

// super user
fn create_products(
    base_product_id: BaseProductId,
    core: &mut tokio_core::reactor::Core,
    http_client: &HttpClientHandle,
    base_url: String,
    user_id: Option<String>,
) -> Result<Products, client::Error> {
    let new_products = NewProducts {
        base_product_id,
        store_id: StoreId(1),
        company_package_id: CompanyPackageId(1),
        price: None,
        deliveries_to: vec![DeliveriesTo {
            country_labels: "rus".to_string().into(),
        }],
    };

    let body: String = serde_json::to_string(&new_products).unwrap().to_string();
    let create_result = core.run(http_client.request_with_auth_header::<Products>(
        Method::Post,
        format!("{}/{}", base_url, MOCK_PRODUCTS_ENDPOINT.to_string()),
        Some(body),
        user_id,
    ));

    create_result
}

// super user
fn delete_products(core: &mut tokio_core::reactor::Core, http_client: &HttpClientHandle, url: String) -> Result<Products, client::Error> {
    let user_id = UserId(1);
    core.run(http_client.request_with_auth_header::<Products>(Method::Delete, url, None, Some(user_id.to_string())))
}

fn get_url_request_by_base_product_id(base_url: String, base_product_id: BaseProductId) -> String {
    format!("{}/{}/{}", base_url, MOCK_PRODUCTS_ENDPOINT, base_product_id)
}

fn get_url_request_by_base_product_id_company_id(base_url: String, base_product_id: BaseProductId, company_package_id: i32) -> String {
    format!(
        "{}/{}/{}/company_package/{}",
        base_url, MOCK_PRODUCTS_ENDPOINT, base_product_id, company_package_id
    )
}

// test products by superuser
#[test]
fn test_products_superuser_crud() {
    let (mut core, http_client) = common::make_utils();
    let base_url = common::setup();
    let user_id = UserId(1);
    let base_product_id = BaseProductId(1);
    let company_package_id = 1;
    let price = 123f64;
    let url_crd = get_url_request_by_base_product_id(base_url.clone(), base_product_id);
    let url_u = get_url_request_by_base_product_id_company_id(base_url.clone(), base_product_id, company_package_id);

    // create
    println!("run create new products for base_product {}", base_product_id);
    let create_result = create_products(
        base_product_id,
        &mut core,
        &http_client,
        base_url.clone(),
        Some(user_id.to_string()),
    );
    println!("{:?}", create_result);
    assert!(create_result.is_ok());

    // read
    println!("run read products for base_product {}", base_product_id);
    let read_result =
        core.run(http_client.request_with_auth_header::<Vec<Products>>(Method::Get, url_crd.clone(), None, Some(user_id.to_string())));
    println!("{:?}", read_result);
    assert!(read_result.is_ok());

    // update
    println!("run update products for base_product {}", base_product_id);
    let update_products = create_update_products(price);
    let update_body: String = serde_json::to_string(&update_products).unwrap().to_string();
    let update_result = core.run(http_client.request_with_auth_header::<Products>(
        Method::Put,
        url_u.clone(),
        Some(update_body),
        Some(user_id.to_string()),
    ));
    println!("{:?}", update_result);
    assert!(update_result.is_ok());

    // delete
    println!("run delete products for base_product {}", base_product_id);
    let delete_result = delete_products(&mut core, &http_client, url_crd.clone());
    assert!(delete_result.is_ok());
}

// test products by regular user
#[test]
fn test_products_regular_user_crud() {
    let (mut core, http_client) = common::make_utils();
    let base_url = common::setup();
    let base_product_id = BaseProductId(2);
    let company_package_id = 1;
    let price = 123f64;
    let url_crd = get_url_request_by_base_product_id(base_url.clone(), base_product_id);
    let url_u = get_url_request_by_base_product_id_company_id(base_url.clone(), base_product_id, company_package_id);

    // create user for test acl
    let user_id = UserId(2);
    let create_role_result = common::create_user_role(user_id.clone(), &mut core, &http_client, base_url.clone());
    assert!(create_role_result.is_ok());

    // create
    println!("run create new products for base_product {} for regular user", base_product_id);
    let create_result = create_products(
        base_product_id,
        &mut core,
        &http_client,
        base_url.clone(),
        Some(user_id.to_string()),
    );
    println!("{:?}", create_result);
    assert!(create_result.is_err());

    // create by super user
    println!("run create new products for base_product {}", base_product_id);
    let create_result = create_products(
        base_product_id,
        &mut core,
        &http_client,
        base_url.clone(),
        Some(UserId(1).to_string()),
    );
    println!("{:?}", create_result);
    assert!(create_result.is_ok());

    // read
    println!("run read products for base_product {}", base_product_id);
    let read_result =
        core.run(http_client.request_with_auth_header::<Vec<Products>>(Method::Get, url_crd.clone(), None, Some(user_id.to_string())));
    println!("{:?}", read_result);
    assert!(read_result.is_ok());

    // update
    println!("run update products for base_product {}", base_product_id);
    let update_products = create_update_products(price);
    let update_body: String = serde_json::to_string(&update_products).unwrap().to_string();
    let update_result = core.run(http_client.request_with_auth_header::<Products>(
        Method::Put,
        url_u.clone(),
        Some(update_body),
        Some(user_id.to_string()),
    ));
    println!("{:?}", update_result);
    assert!(update_result.is_err());

    // delete by super for test
    println!("run delete products for base_product {}", base_product_id);
    let delete_result = delete_products(&mut core, &http_client, url_crd.clone());
    assert!(delete_result.is_ok());

    // delete user role
    let delete_result = common::delete_role(user_id.clone(), &mut core, &http_client, base_url.clone());
    assert!(delete_result.is_ok());
}

// test update products without authorization data
#[test]
fn test_products_unauthorized() {
    let (mut core, http_client) = common::make_utils();
    let base_url = common::setup();
    let base_product_id = BaseProductId(3);
    let company_package_id = 1;
    let price = 123f64;
    let url_crd = get_url_request_by_base_product_id(base_url.clone(), base_product_id);
    let url_u = get_url_request_by_base_product_id_company_id(base_url.clone(), base_product_id, company_package_id);

    // create
    println!("run create new products for base_product {}", base_product_id);
    let create_result = create_products(base_product_id, &mut core, &http_client, base_url.clone(), None);
    println!("{:?}", create_result);
    assert!(create_result.is_err());

    // create by super user
    println!("run create new products for base_product {}", base_product_id);
    let create_result = create_products(
        base_product_id,
        &mut core,
        &http_client,
        base_url.clone(),
        Some(UserId(1).to_string()),
    );
    println!("{:?}", create_result);
    assert!(create_result.is_ok());

    // read
    println!("run read products for base_product {}", base_product_id);
    let read_result = core.run(http_client.request_with_auth_header::<Vec<Products>>(Method::Get, url_crd.clone(), None, None));
    println!("{:?}", read_result);
    assert!(read_result.is_err());

    // update
    println!("run update products for base_product {}", base_product_id);
    let update_products = create_update_products(price);
    let update_body: String = serde_json::to_string(&update_products).unwrap().to_string();
    let update_result = core.run(http_client.request_with_auth_header::<Products>(Method::Put, url_u.clone(), Some(update_body), None));
    println!("{:?}", update_result);
    assert!(update_result.is_err());

    // delete by super for test
    println!("run delete products for base_product {}", base_product_id);
    let delete_result = delete_products(&mut core, &http_client, url_crd.clone());
    assert!(delete_result.is_ok());
}

// test products by store manager
#[test]
fn test_products_store_manager() {
    let (mut core, http_client) = common::make_utils();
    let base_url = common::setup();
    let base_product_id = BaseProductId(4);
    let store_id = StoreId(1);
    let company_package_id = 1;
    let price = 123f64;
    let url_crd = get_url_request_by_base_product_id(base_url.clone(), base_product_id);
    let url_u = get_url_request_by_base_product_id_company_id(base_url.clone(), base_product_id, company_package_id);

    // create store_manager for test acl
    let user_id = UserId(3);
    let create_role_result = common::create_user_store_role(user_id.clone(), store_id, &mut core, &http_client, base_url.clone());
    assert!(create_role_result.is_ok());

    // create
    println!("run create new products for base_product {}", base_product_id);
    let create_result = create_products(
        base_product_id,
        &mut core,
        &http_client,
        base_url.clone(),
        Some(user_id.to_string()),
    );
    println!("{:?}", create_result);
    assert!(create_result.is_ok());

    // read
    println!("run read products for base_product {}", base_product_id);
    let read_result =
        core.run(http_client.request_with_auth_header::<Vec<Products>>(Method::Get, url_crd.clone(), None, Some(user_id.to_string())));
    println!("{:?}", read_result);
    assert!(read_result.is_ok());

    // update
    println!("run update products for base_product {}", base_product_id);
    let update_products = create_update_products(price);
    let update_body: String = serde_json::to_string(&update_products).unwrap().to_string();
    let update_result = core.run(http_client.request_with_auth_header::<Products>(
        Method::Put,
        url_u.clone(),
        Some(update_body),
        Some(user_id.to_string()),
    ));
    println!("{:?}", update_result);
    assert!(update_result.is_ok());

    // delete by super for test
    println!("run delete products for base_product {}", base_product_id);
    let delete_result = delete_products(&mut core, &http_client, url_crd.clone());
    assert!(delete_result.is_ok());

    // delete user role
    let delete_result = common::delete_role(user_id.clone(), &mut core, &http_client, base_url.clone());
    assert!(delete_result.is_ok());
}
