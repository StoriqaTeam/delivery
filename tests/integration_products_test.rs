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

// super user
fn create_shipping(
    base_product_id: BaseProductId,
    company_package_id: CompanyPackageId,
    store_id: StoreId,
    deliveries_to: Vec<Alpha3>,
    core: &mut tokio_core::reactor::Core,
    http_client: &HttpClientHandle,
    base_url: String,
    user_id: Option<String>,
) -> Result<Shipping, client::Error> {
    let new_product = NewProducts {
        base_product_id: base_product_id.clone(),
        store_id: store_id.clone(),
        company_package_id,
        price: None,
        shipping: ShippingVariant::Local,
        deliveries_to,
    };

    let new_pickup = NewPickups {
        base_product_id,
        store_id,
        pickup: true,
        price: None,
    };

    let shipping = NewShipping {
        items: vec![new_product],
        pickup: Some(new_pickup),
    };

    let body: String = serde_json::to_string(&shipping).unwrap().to_string();
    let create_result = core.run(http_client.request_with_auth_header::<Shipping>(Method::Post, base_url, Some(body), user_id));

    create_result
}

// super user
fn delete_products(core: &mut tokio_core::reactor::Core, http_client: &HttpClientHandle, url: String) -> Result<(), client::Error> {
    let user_id = UserId(1);
    core.run(http_client.request_with_auth_header::<()>(Method::Delete, url, None, Some(user_id.to_string())))
}

fn get_url_request_by_base_product_id(base_url: String, base_product_id: BaseProductId) -> String {
    format!("{}/{}/{}", base_url, MOCK_PRODUCTS_ENDPOINT, base_product_id)
}

#[test]
fn test_company() {
    let (mut core, http_client) = common::make_utils();
    let base_url = common::setup();

    test_products_superuser_crud(&mut core, &http_client, base_url.clone());
    test_products_regular_user_crud(&mut core, &http_client, base_url.clone());
    test_products_unauthorized(&mut core, &http_client, base_url.clone());
    test_products_store_manager(&mut core, &http_client, base_url.clone());
}

fn create_company(name: String) -> NewCompany {
    NewCompany {
        name,
        label: "UPS".to_string(),
        description: None,
        deliveries_from: vec![Alpha3("RUS".to_string())],
        logo: "".to_string(),
    }
}

fn create_package(name: String) -> NewPackages {
    NewPackages {
        name,
        max_size: 0f64,
        min_size: 0f64,
        max_weight: 0f64,
        min_weight: 0f64,
        deliveries_to: vec![],
    }
}

// test products by superuser
fn test_products_superuser_crud(core: &mut tokio_core::reactor::Core, http_client: &HttpClientHandle, base_url: String) {
    let user_id = UserId(1);
    let base_product_id = BaseProductId(1);
    let store_id = StoreId(1);
    let url_crd = get_url_request_by_base_product_id(base_url.clone(), base_product_id);

    let package_name = "Avia".to_string();
    let company_name = "US UPS".to_string();

    let countries = vec!["RUS", "USA", "BRA"].into_iter().map(|v| Alpha3(v.to_string())).collect();

    let payload = (create_company(company_name), create_package(package_name));
    let (package_id, company_id, companies_package_id) =
        common::create_delivery_objects(payload, core, http_client, base_url.clone(), Some(user_id));

    // upsert
    println!("run upsert products for base_product {}", base_product_id);
    let create_result = create_shipping(
        base_product_id,
        companies_package_id.clone(),
        store_id,
        countries,
        core,
        http_client,
        url_crd.clone(),
        Some(user_id.to_string()),
    );
    println!("create shipping {:?}", create_result);
    assert!(create_result.is_ok());

    common::delete_deliveries_objects(
        (package_id.clone(), company_id.clone(), companies_package_id.clone()),
        core,
        http_client,
        base_url.clone(),
        user_id,
    );

    // delete
    println!("run delete products for base_product {}", base_product_id);
    let delete_result = delete_products(core, http_client, url_crd.clone());
    assert!(delete_result.is_ok());
}

// test products by regular user
fn test_products_regular_user_crud(core: &mut tokio_core::reactor::Core, http_client: &HttpClientHandle, base_url: String) {
    let base_product_id = BaseProductId(2);
    let store_id = StoreId(1);
    let package_name = "Avia".to_string();
    let company_name = "US UPS".to_string();
    let super_user_id = UserId(1);

    let url_crd = get_url_request_by_base_product_id(base_url.clone(), base_product_id);

    // create user for test acl
    let user_id = UserId(2);
    let create_role_result = common::create_user_role(user_id.clone(), core, http_client, base_url.clone());
    assert!(create_role_result.is_ok());

    let countries = vec!["RUS", "USA", "BRA"].into_iter().map(|v| Alpha3(v.to_string())).collect();

    let payload = (create_company(company_name), create_package(package_name));
    let (package_id, company_id, companies_package_id) =
        common::create_delivery_objects(payload, core, http_client, base_url.clone(), Some(super_user_id));

    // upsert
    println!("run upsert products for base_product {}", base_product_id);
    let create_result = create_shipping(
        base_product_id,
        companies_package_id.clone(),
        store_id,
        countries,
        core,
        http_client,
        url_crd.clone(),
        Some(user_id.to_string()),
    );
    println!("create shipping {:?}", create_result);
    assert!(create_result.is_err());

    common::delete_deliveries_objects(
        (package_id.clone(), company_id.clone(), companies_package_id.clone()),
        core,
        http_client,
        base_url.clone(),
        super_user_id.clone(),
    );

    // delete
    println!("run delete products for base_product {}", base_product_id);
    let delete_result = delete_products(core, http_client, url_crd.clone());
    assert!(delete_result.is_ok());

    // delete user role
    let delete_result = common::delete_role(user_id.clone(), core, http_client, base_url.clone());
    assert!(delete_result.is_ok());
}

// test update products without authorization data
fn test_products_unauthorized(core: &mut tokio_core::reactor::Core, http_client: &HttpClientHandle, base_url: String) {
    let base_product_id = BaseProductId(3);
    let store_id = StoreId(1);
    let package_name = "Avia".to_string();
    let company_name = "US UPS".to_string();
    let super_user_id = UserId(1);

    let url_crd = get_url_request_by_base_product_id(base_url.clone(), base_product_id);

    let countries = vec!["RUS", "USA", "BRA"].into_iter().map(|v| Alpha3(v.to_string())).collect();

    let payload = (create_company(company_name), create_package(package_name));
    let (package_id, company_id, companies_package_id) =
        common::create_delivery_objects(payload, core, http_client, base_url.clone(), Some(super_user_id.clone()));

    // upsert
    println!("run upsert products for base_product {}", base_product_id);
    let create_result = create_shipping(
        base_product_id,
        companies_package_id.clone(),
        store_id,
        countries,
        core,
        http_client,
        url_crd.clone(),
        None,
    );
    println!("create shipping {:?}", create_result);
    assert!(create_result.is_err());

    common::delete_deliveries_objects(
        (package_id.clone(), company_id.clone(), companies_package_id.clone()),
        core,
        http_client,
        base_url.clone(),
        super_user_id.clone(),
    );

    // delete
    println!("run delete products for base_product {}", base_product_id);
    let delete_result = delete_products(core, http_client, url_crd.clone());
    assert!(delete_result.is_ok());
}

// test products by store manager
fn test_products_store_manager(core: &mut tokio_core::reactor::Core, http_client: &HttpClientHandle, base_url: String) {
    let base_product_id = BaseProductId(4);
    let store_id = StoreId(1);
    let package_name = "Avia".to_string();
    let company_name = "US UPS".to_string();
    let super_user_id = UserId(1);
    let url_crd = get_url_request_by_base_product_id(base_url.clone(), base_product_id);

    // create store_manager for test acl
    let user_id = UserId(3);
    let create_role_result = common::create_user_role(user_id.clone(), core, http_client, base_url.clone());
    assert!(create_role_result.is_ok());
    let create_role_result = common::create_user_store_role(user_id.clone(), store_id, core, http_client, base_url.clone());
    println!("traceeee {:?}", create_role_result);
    assert!(create_role_result.is_ok());

    let countries = vec!["RUS", "USA", "BRA"].into_iter().map(|v| Alpha3(v.to_string())).collect();

    let payload = (create_company(company_name), create_package(package_name));
    let (package_id, company_id, companies_package_id) =
        common::create_delivery_objects(payload, core, http_client, base_url.clone(), Some(super_user_id));

    // upsert
    println!("run upsert products for base_product {}", base_product_id);
    let create_result = create_shipping(
        base_product_id,
        companies_package_id.clone(),
        store_id,
        countries,
        core,
        http_client,
        url_crd.clone(),
        Some(user_id.to_string()),
    );
    println!("create shipping {:?}", create_result);
    assert!(create_result.is_ok());

    common::delete_deliveries_objects(
        (package_id.clone(), company_id.clone(), companies_package_id.clone()),
        core,
        http_client,
        base_url.clone(),
        super_user_id.clone(),
    );

    // delete
    println!("run delete products for base_product {}", base_product_id);
    let delete_result = delete_products(core, http_client, url_crd.clone());
    assert!(delete_result.is_ok());

    // delete user role
    let delete_result = common::delete_role(user_id.clone(), core, http_client, base_url.clone());
    assert!(delete_result.is_ok());
}
