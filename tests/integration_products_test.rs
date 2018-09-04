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

static MOCK_COMPANIES_PACKAGES_ENDPOINT: &'static str = "companies_packages";
static MOCK_COMPANIES_ENDPOINT: &'static str = "companies";
static MOCK_PACKAGES_ENDPOINT: &'static str = "packages";
static MOCK_PRODUCTS_ENDPOINT: &'static str = "products";
static MOCK_COUNTRY_ENDPOINT: &'static str = "countries";

fn create_companies_packages(
    company_id: CompanyId,
    package_id: PackageId,
    core: &mut tokio_core::reactor::Core,
    http_client: &HttpClientHandle,
    base_url: String,
    user_id: Option<UserId>,
) -> Result<CompaniesPackages, client::Error> {
    let new_companies_packages = NewCompaniesPackages { company_id, package_id };

    let body: String = serde_json::to_string(&new_companies_packages).unwrap().to_string();
    let create_result = core.run(http_client.request_with_auth_header::<CompaniesPackages>(
        Method::Post,
        format!("{}/{}", base_url, MOCK_COMPANIES_PACKAGES_ENDPOINT),
        Some(body),
        user_id.map(|u| u.to_string()),
    ));

    create_result
}

fn create_company(
    name: String,
    core: &mut tokio_core::reactor::Core,
    http_client: &HttpClientHandle,
    base_url: String,
    user_id: Option<UserId>,
) -> Result<Company, client::Error> {
    let new_company = NewCompany {
        name,
        label: "UPS".to_string(),
        description: None,
        deliveries_from: vec![CountryLabel("rus".to_string())],
        logo: "".to_string(),
    };

    let body: String = serde_json::to_string(&new_company).unwrap().to_string();
    let create_result = core.run(http_client.request_with_auth_header::<Company>(
        Method::Post,
        format!("{}/{}", base_url, MOCK_COMPANIES_ENDPOINT),
        Some(body),
        user_id.map(|u| u.to_string()),
    ));

    create_result
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
        max_size: 0f64,
        min_size: 0f64,
        max_weight: 0f64,
        min_weight: 0f64,
        deliveries_to: vec![],
    };

    let body: String = serde_json::to_string(&new).unwrap().to_string();
    let create_result = core.run(http_client.request_with_auth_header::<Packages>(
        Method::Post,
        format!("{}/{}", base_url, MOCK_PACKAGES_ENDPOINT),
        Some(body),
        user_id.map(|u| u.to_string()),
    ));

    create_result
}

fn get_all_countries(
    core: &mut tokio_core::reactor::Core,
    http_client: &HttpClientHandle,
    base_url: String,
    user_id: Option<UserId>,
) -> Result<Country, client::Error> {
    let create_result = core.run(http_client.request_with_auth_header::<Country>(
        Method::Get,
        format!("{}/{}", base_url, MOCK_COUNTRY_ENDPOINT.to_string()),
        None,
        user_id.map(|u| u.to_string()),
    ));

    create_result
}

// super user
fn create_shipping(
    base_product_id: BaseProductId,
    company_package_id: CompanyPackageId,
    deliveries_to: Vec<Country>,
    core: &mut tokio_core::reactor::Core,
    http_client: &HttpClientHandle,
    base_url: String,
    user_id: Option<String>,
) -> Result<Shipping, client::Error> {
    let store_id = StoreId(1);

    let new_product = NewProducts {
        base_product_id: base_product_id.clone(),
        store_id: store_id.clone(),
        company_package_id,
        price: None,
        shipping: ShippingVariant::Local,
    };

    let shipping_products = NewShippingProducts {
        product: new_product,
        deliveries_to,
    };

    let new_pickup = NewPickups {
        base_product_id,
        store_id,
        pickup: true,
        price: None,
    };

    let shipping = NewShipping {
        items: vec![shipping_products],
        pickup: Some(new_pickup),
    };

    let body: String = serde_json::to_string(&shipping).unwrap().to_string();
    let create_result = core.run(http_client.request_with_auth_header::<Shipping>(Method::Post, base_url, Some(body), user_id));

    create_result
}

fn create_delivery_objects(
    package_name: String,
    company_name: String,
    core: &mut tokio_core::reactor::Core,
    http_client: &HttpClientHandle,
    base_url: String,
    user_id: Option<UserId>,
) -> (PackageId, CompanyId, CompanyPackageId) {
    // create
    println!("run create new package ");
    let create_result = create_package(package_name.clone(), core, &http_client, base_url.clone(), user_id);
    println!("{:?}", create_result);
    assert!(create_result.is_ok());
    let package_id = create_result.unwrap().id;

    // create
    println!("run create new company ");
    let create_result = create_company(company_name.clone(), core, &http_client, base_url.clone(), user_id);
    println!("{:?}", create_result);
    assert!(create_result.is_ok());
    let company_id = create_result.unwrap().id;

    // create
    println!("run create new companies_packages ");
    let create_result = create_companies_packages(company_id, package_id, core, &http_client, base_url.clone(), user_id);
    println!("{:?}", create_result);
    assert!(create_result.is_ok());
    let companies_package_id = create_result.unwrap().id;

    (package_id, company_id, companies_package_id)
}

fn delete_deliveries_objects(
    core: &mut tokio_core::reactor::Core,
    http_client: &HttpClientHandle,
    base_url: String,
    user_id: UserId,
    ids: (PackageId, CompanyId, CompanyPackageId),
) {
    let (package_id, company_id, companies_package_id) = ids;

    println!("run delete companies_packages ");
    let delete_result = core.run(http_client.request_with_auth_header::<CompaniesPackages>(
        Method::Delete,
        format!("{}/{}/{}", base_url, MOCK_COMPANIES_PACKAGES_ENDPOINT, companies_package_id),
        None,
        Some(user_id.to_string()),
    ));
    assert!(delete_result.is_ok());

    println!("run delete company ");
    let delete_result = core.run(http_client.request_with_auth_header::<Company>(
        Method::Delete,
        format!("{}/{}/{}", base_url, MOCK_COMPANIES_ENDPOINT, company_id),
        None,
        Some(user_id.to_string()),
    ));
    assert!(delete_result.is_ok());

    println!("run delete package ");
    let delete_result = core.run(http_client.request_with_auth_header::<Packages>(
        Method::Delete,
        format!("{}/{}/{}", base_url, MOCK_PACKAGES_ENDPOINT, package_id),
        None,
        Some(user_id.to_string()),
    ));
    assert!(delete_result.is_ok());
}

// super user
fn delete_products(core: &mut tokio_core::reactor::Core, http_client: &HttpClientHandle, url: String) -> Result<(), client::Error> {
    let user_id = UserId(1);
    core.run(http_client.request_with_auth_header::<()>(Method::Delete, url, None, Some(user_id.to_string())))
}

fn get_url_request_by_base_product_id(base_url: String, base_product_id: BaseProductId) -> String {
    format!("{}/{}/{}", base_url, MOCK_PRODUCTS_ENDPOINT, base_product_id)
}

// test products by superuser
#[test]
fn test_products_superuser_crud() {
    let (mut core, http_client) = common::make_utils();
    let base_url = common::setup();
    let user_id = UserId(1);
    let base_product_id = BaseProductId(1);
    let url_crd = get_url_request_by_base_product_id(base_url.clone(), base_product_id);

    let package_name = "Avia".to_string();
    let company_name = "US UPS".to_string();

    println!("run get_all countries");
    let countries_result = get_all_countries(&mut core, &http_client, base_url.clone(), Some(user_id));
    assert!(countries_result.is_ok());
    let countries = vec![countries_result.unwrap()];

    let (package_id, company_id, companies_package_id) = create_delivery_objects(
        package_name.clone(),
        company_name.clone(),
        &mut core,
        &http_client,
        base_url.clone(),
        Some(user_id),
    );

    // upsert
    println!("run upsert products for base_product {}", base_product_id);
    let create_result = create_shipping(
        base_product_id,
        companies_package_id.clone(),
        countries,
        &mut core,
        &http_client,
        url_crd.clone(),
        Some(user_id.to_string()),
    );
    println!("create shipping {:?}", create_result);
    assert!(create_result.is_ok());

    delete_deliveries_objects(
        &mut core,
        &http_client,
        base_url.clone(),
        user_id,
        (package_id.clone(), company_id.clone(), companies_package_id.clone()),
    );

    // delete
    println!("run delete products for base_product {}", base_product_id);
    let delete_result = delete_products(&mut core, &http_client, url_crd.clone());
    assert!(delete_result.is_ok());
}

// test products by regular user
#[ignore]
#[test]
fn test_products_regular_user_crud() {
    let (mut core, http_client) = common::make_utils();
    let base_url = common::setup();
    let base_product_id = BaseProductId(2);
    let package_name = "Avia".to_string();
    let company_name = "US UPS".to_string();
    let super_user_id = UserId(1);

    let url_crd = get_url_request_by_base_product_id(base_url.clone(), base_product_id);

    // create user for test acl
    let user_id = UserId(2);
    let create_role_result = common::create_user_role(user_id.clone(), &mut core, &http_client, base_url.clone());
    assert!(create_role_result.is_ok());

    println!("run get_all countries");
    let countries_result = get_all_countries(&mut core, &http_client, base_url.clone(), Some(user_id));
    assert!(countries_result.is_ok());
    let countries = vec![countries_result.unwrap()];

    let (package_id, company_id, companies_package_id) = create_delivery_objects(
        package_name.clone(),
        company_name.clone(),
        &mut core,
        &http_client,
        base_url.clone(),
        Some(super_user_id),
    );

    // upsert
    println!("run upsert products for base_product {}", base_product_id);
    let create_result = create_shipping(
        base_product_id,
        companies_package_id.clone(),
        countries,
        &mut core,
        &http_client,
        url_crd.clone(),
        Some(user_id.to_string()),
    );
    println!("create shipping {:?}", create_result);
    assert!(create_result.is_err());

    delete_deliveries_objects(
        &mut core,
        &http_client,
        base_url.clone(),
        super_user_id.clone(),
        (package_id.clone(), company_id.clone(), companies_package_id.clone()),
    );

    // delete
    println!("run delete products for base_product {}", base_product_id);
    let delete_result = delete_products(&mut core, &http_client, url_crd.clone());
    assert!(delete_result.is_ok());

    // delete user role
    let delete_result = common::delete_role(user_id.clone(), &mut core, &http_client, base_url.clone());
    assert!(delete_result.is_ok());
}

// test update products without authorization data
#[ignore]
#[test]
fn test_products_unauthorized() {
    let (mut core, http_client) = common::make_utils();
    let base_url = common::setup();
    let base_product_id = BaseProductId(3);
    let package_name = "Avia".to_string();
    let company_name = "US UPS".to_string();
    let super_user_id = UserId(1);

    let url_crd = get_url_request_by_base_product_id(base_url.clone(), base_product_id);

    println!("run get_all countries");
    let countries_result = get_all_countries(&mut core, &http_client, base_url.clone(), Some(super_user_id));
    assert!(countries_result.is_ok());
    let countries = vec![countries_result.unwrap()];

    let (package_id, company_id, companies_package_id) = create_delivery_objects(
        package_name.clone(),
        company_name.clone(),
        &mut core,
        &http_client,
        base_url.clone(),
        Some(super_user_id),
    );

    // upsert
    println!("run upsert products for base_product {}", base_product_id);
    let create_result = create_shipping(
        base_product_id,
        companies_package_id.clone(),
        countries,
        &mut core,
        &http_client,
        url_crd.clone(),
        None,
    );
    println!("create shipping {:?}", create_result);
    assert!(create_result.is_err());

    delete_deliveries_objects(
        &mut core,
        &http_client,
        base_url.clone(),
        super_user_id.clone(),
        (package_id.clone(), company_id.clone(), companies_package_id.clone()),
    );

    // delete
    println!("run delete products for base_product {}", base_product_id);
    let delete_result = delete_products(&mut core, &http_client, url_crd.clone());
    assert!(delete_result.is_ok());
}

// test products by store manager
#[ignore]
#[test]
fn test_products_store_manager() {
    let (mut core, http_client) = common::make_utils();
    let base_url = common::setup();
    let base_product_id = BaseProductId(4);
    let store_id = StoreId(1);
    let package_name = "Avia".to_string();
    let company_name = "US UPS".to_string();
    let super_user_id = UserId(1);
    let url_crd = get_url_request_by_base_product_id(base_url.clone(), base_product_id);

    // create store_manager for test acl
    let user_id = UserId(3);
    let create_role_result = common::create_user_role(user_id.clone(), &mut core, &http_client, base_url.clone());
    assert!(create_role_result.is_ok());
    let create_role_result = common::create_user_store_role(user_id.clone(), store_id, &mut core, &http_client, base_url.clone());
    assert!(create_role_result.is_ok());

    println!("run get_all countries");
    let countries_result = get_all_countries(&mut core, &http_client, base_url.clone(), Some(user_id));
    assert!(countries_result.is_ok());
    let countries = vec![countries_result.unwrap()];

    let (package_id, company_id, companies_package_id) = create_delivery_objects(
        package_name.clone(),
        company_name.clone(),
        &mut core,
        &http_client,
        base_url.clone(),
        Some(super_user_id),
    );

    // upsert
    println!("run upsert products for base_product {}", base_product_id);
    let create_result = create_shipping(
        base_product_id,
        companies_package_id.clone(),
        countries,
        &mut core,
        &http_client,
        url_crd.clone(),
        Some(user_id.to_string()),
    );
    println!("create shipping {:?}", create_result);
    assert!(create_result.is_ok());

    delete_deliveries_objects(
        &mut core,
        &http_client,
        base_url.clone(),
        super_user_id.clone(),
        (package_id.clone(), company_id.clone(), companies_package_id.clone()),
    );

    // delete
    println!("run delete products for base_product {}", base_product_id);
    let delete_result = delete_products(&mut core, &http_client, url_crd.clone());
    assert!(delete_result.is_ok());

    // delete user role
    let delete_result = common::delete_role(user_id.clone(), &mut core, &http_client, base_url.clone());
    assert!(delete_result.is_ok());
}
