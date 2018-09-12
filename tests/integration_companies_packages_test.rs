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

static MOCK_COMPANIES_PACKAGES_ENDPOINT: &'static str = "companies_packages";
static MOCK_COMPANIES_ENDPOINT: &'static str = "companies";
static MOCK_PACKAGES_ENDPOINT: &'static str = "packages";

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
        get_url_request(base_url),
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
        deliveries_from: vec![Alpha3("RUS".to_string())],
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
        deliveries_to: vec![Alpha3("USA".to_string()), Alpha3("CHN".to_string())],
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

fn get_url_request_by_id(base_url: String, companies_packages_id: CompanyPackageId) -> String {
    format!("{}/{}/{}", base_url, MOCK_COMPANIES_PACKAGES_ENDPOINT, companies_packages_id)
}

fn get_url_request_by_company_id(base_url: String, company_id: CompanyId) -> String {
    format!("{}/companies/{}/packages", base_url, company_id)
}

fn get_url_request_by_package_id(base_url: String, package_id: PackageId) -> String {
    format!("{}/packages/{}/companies", base_url, package_id)
}

fn get_url_request(base_url: String) -> String {
    format!("{}/{}", base_url, MOCK_COMPANIES_PACKAGES_ENDPOINT)
}

fn get_url_request_available_packages(base_url: String, country: Alpha3, size: f64, weight: f64) -> String {
    format!(
        "{}/available_packages?country={}&size={}&weight={}",
        base_url, country.0, size, weight
    )
}

#[test]
fn test_companies_packages() {
    let (mut core, http_client) = common::make_utils();
    let base_url = common::setup();

    test_companies_packages_superuser_crud(&mut core, &http_client, base_url.clone());
    test_companies_packages_regular_user_crud(&mut core, &http_client, base_url.clone());
    test_companies_packages_unauthorized(&mut core, &http_client, base_url.clone());
}

// test companies_packages by superuser
fn test_companies_packages_superuser_crud(core: &mut tokio_core::reactor::Core, http_client: &HttpClientHandle, base_url: String) {
    let user_id = UserId(1);
    let package_name = "Avia".to_string();
    let company_name = "US UPS".to_string();

    // create
    println!("user: {:?} - run create new package", user_id);
    let create_result = create_package(package_name.clone(), core, &http_client, base_url.clone(), Some(user_id));
    println!("user: {:?} - new package {:?}", user_id, create_result);
    assert!(create_result.is_ok(), "not create package!");
    let package_id = create_result.unwrap().id;

    // create
    println!("user: {:?} - run create new company", user_id);
    let create_result = create_company(company_name.clone(), core, &http_client, base_url.clone(), Some(user_id));
    println!("user: {:?} - new company {:?}", user_id, create_result);
    assert!(create_result.is_ok());
    let company_id = create_result.unwrap().id;

    // create
    println!("user: {:?} - run create new companies_packages", user_id);
    let create_result = create_companies_packages(company_id, package_id, core, &http_client, base_url.clone(), Some(user_id));
    println!("user: {:?} - new companies_packages{:?}", user_id, create_result);
    assert!(create_result.is_ok());
    let companies_packages = create_result.unwrap();

    // read search
    println!(
        "user: {:?} - run search companies_packages by id {:?}",
        user_id, companies_packages.id
    );
    let read_result = core.run(http_client.request_with_auth_header::<CompaniesPackages>(
        Method::Get,
        get_url_request_by_id(base_url.clone(), companies_packages.id.clone()),
        None,
        Some(user_id.to_string()),
    ));
    println!(
        "user: {:?} - find companies_packages {:?} by id {}",
        user_id, read_result, companies_packages.id
    );
    assert!(read_result.is_ok());

    // read companies
    println!("user: {:?} - run search companies by package id {:?}", user_id, package_id);
    let read_result = core.run(http_client.request_with_auth_header::<Vec<Company>>(
        Method::Get,
        get_url_request_by_package_id(base_url.clone(), package_id),
        None,
        Some(user_id.to_string()),
    ));
    println!(
        "user: {:?} - find companies by package id {:?} by id {:?}",
        user_id, read_result, package_id
    );
    assert!(read_result.is_ok());

    // read packages
    println!("user: {:?} - run search packages by company id {:?}", user_id, company_id);
    let read_result = core.run(http_client.request_with_auth_header::<Vec<Packages>>(
        Method::Get,
        get_url_request_by_company_id(base_url.clone(), company_id),
        None,
        Some(user_id.to_string()),
    ));
    println!(
        "user: {:?} - find packages by company id {:?} by id {:?}",
        user_id, read_result, company_id
    );
    assert!(read_result.is_ok());

    // read available packages
    let country_search = Alpha3("RUS".to_string());
    println!(
        "user: {:?} - run search available packages by country {:?}",
        user_id, country_search
    );
    let read_result = core.run(http_client.request_with_auth_header::<Vec<AvailablePackages>>(
        Method::Get,
        get_url_request_available_packages(base_url.clone(), country_search, 0f64, 0f64),
        None,
        Some(user_id.to_string()),
    ));
    println!("user: {:?} - find available packages {:#?}", user_id, read_result);
    assert!(read_result.is_ok());

    // delete
    println!("user: {:?} - run delete companies_packages", user_id);
    let delete_result = core.run(http_client.request_with_auth_header::<CompaniesPackages>(
        Method::Delete,
        get_url_request_by_id(base_url.clone(), companies_packages.id),
        None,
        Some(user_id.to_string()),
    ));
    assert!(delete_result.is_ok());

    let user_id = UserId(1);
    // delete
    println!("user: {:?} - run delete company", user_id);
    let delete_result = core.run(http_client.request_with_auth_header::<Company>(
        Method::Delete,
        format!("{}/{}/{}", base_url, MOCK_COMPANIES_ENDPOINT, company_id),
        None,
        Some(user_id.to_string()),
    ));
    assert!(delete_result.is_ok());

    // delete
    println!("user: {:?} - run delete package", user_id);
    let delete_result = core.run(http_client.request_with_auth_header::<Packages>(
        Method::Delete,
        format!("{}/{}/{}", base_url, MOCK_PACKAGES_ENDPOINT, package_id),
        None,
        Some(user_id.to_string()),
    ));
    assert!(delete_result.is_ok());
}

// test companies_packages by regular user
fn test_companies_packages_regular_user_crud(core: &mut tokio_core::reactor::Core, http_client: &HttpClientHandle, base_url: String) {
    let package_name = "Avia".to_string();
    let company_name = "US UPS".to_string();
    let user_id = UserId(1);

    // create
    println!("run create new package ");
    let create_result = create_package(package_name.clone(), core, &http_client, base_url.clone(), Some(user_id));
    println!("{:?}", create_result);
    assert!(create_result.is_ok());
    let package_id = create_result.unwrap().id;
    // create
    println!("run create new company ");
    let create_result = create_company(company_name.clone(), core, &http_client, base_url.clone(), Some(user_id));
    println!("{:?}", create_result);
    assert!(create_result.is_ok());
    let company_id = create_result.unwrap().id;

    // create user for test acl
    let user_id = UserId(1123);
    let create_role_result = common::create_user_role(user_id.clone(), core, &http_client, base_url.clone());
    assert!(create_role_result.is_ok());

    // create
    println!("run create new companies_packages ");
    let create_result = create_companies_packages(company_id, package_id, core, &http_client, base_url.clone(), Some(user_id));
    println!("{:?}", create_result);
    assert!(create_result.is_err());

    // create by super user
    println!("run create new companies_packages by super user");
    let create_result = create_companies_packages(company_id, package_id, core, &http_client, base_url.clone(), Some(UserId(1)));
    println!("{:?}", create_result);
    assert!(create_result.is_ok());

    let companies_packages = create_result.unwrap();
    // read search
    println!("run search companies_packages by id");
    let read_result = core.run(http_client.request_with_auth_header::<CompaniesPackages>(
        Method::Get,
        get_url_request_by_id(base_url.clone(), companies_packages.id),
        None,
        Some(user_id.to_string()),
    ));
    println!("{:?}", read_result);
    assert!(read_result.is_ok());

    // read companies
    println!("run search companies by package id");
    let read_result = core.run(http_client.request_with_auth_header::<Vec<Company>>(
        Method::Get,
        get_url_request_by_package_id(base_url.clone(), package_id),
        None,
        Some(user_id.to_string()),
    ));
    println!("companies by package id {:?}", read_result);
    assert!(read_result.is_ok());

    // read packages
    println!("run search packages by company id");
    let read_result = core.run(http_client.request_with_auth_header::<Vec<Packages>>(
        Method::Get,
        get_url_request_by_company_id(base_url.clone(), company_id),
        None,
        Some(user_id.to_string()),
    ));
    println!("packages by company id {:?}", read_result);
    assert!(read_result.is_ok());

    // read available packages
    let country_search = Alpha3("RUS".to_string());
    println!(
        "user: {:?} - run search available packages by country {:?}",
        user_id, country_search
    );
    let read_result = core.run(http_client.request_with_auth_header::<Vec<AvailablePackages>>(
        Method::Get,
        get_url_request_available_packages(base_url.clone(), country_search, 0f64, 0f64),
        None,
        Some(user_id.to_string()),
    ));
    println!("user: {:?} - find available packages {:#?}", user_id, read_result);
    assert!(read_result.is_ok());

    // delete
    println!("run delete companies_packages ");
    let delete_result = core.run(http_client.request_with_auth_header::<CompaniesPackages>(
        Method::Delete,
        get_url_request_by_id(base_url.clone(), companies_packages.id),
        None,
        Some(user_id.to_string()),
    ));
    assert!(delete_result.is_err());

    // delete by super user
    println!("run delete companies_packages by super user ");
    let delete_result = core.run(http_client.request_with_auth_header::<CompaniesPackages>(
        Method::Delete,
        get_url_request_by_id(base_url.clone(), companies_packages.id),
        None,
        Some("1".to_string()),
    ));
    assert!(delete_result.is_ok());

    let user_id = UserId(1);
    // delete
    println!("run delete company ");
    let delete_result = core.run(http_client.request_with_auth_header::<Company>(
        Method::Delete,
        format!("{}/{}/{}", base_url, MOCK_COMPANIES_ENDPOINT, company_id),
        None,
        Some(user_id.to_string()),
    ));
    assert!(delete_result.is_ok());

    // delete
    println!("run delete package ");
    let delete_result = core.run(http_client.request_with_auth_header::<Packages>(
        Method::Delete,
        format!("{}/{}/{}", base_url, MOCK_PACKAGES_ENDPOINT, package_id),
        None,
        Some(user_id.to_string()),
    ));
    assert!(delete_result.is_ok());
}

// test update companies_packages without authorization data
fn test_companies_packages_unauthorized(core: &mut tokio_core::reactor::Core, http_client: &HttpClientHandle, base_url: String) {
    let package_name = "Avia".to_string();
    let company_name = "US UPS".to_string();
    let user_id = UserId(1);

    // create
    println!("run create new package ");
    let create_result = create_package(package_name.clone(), core, &http_client, base_url.clone(), Some(user_id));
    println!("{:?}", create_result);
    assert!(create_result.is_ok());
    let package_id = create_result.unwrap().id;
    // create
    println!("run create new company ");
    let create_result = create_company(company_name.clone(), core, &http_client, base_url.clone(), Some(user_id));
    println!("{:?}", create_result);
    assert!(create_result.is_ok());
    let company_id = create_result.unwrap().id;

    // create
    println!("run create new companies_packages ");
    let create_result = create_companies_packages(company_id, package_id, core, &http_client, base_url.clone(), None);
    println!("{:?}", create_result);
    assert!(create_result.is_err());

    // create by super user
    println!("run create new companies_packages by super user");
    let create_result = create_companies_packages(company_id, package_id, core, &http_client, base_url.clone(), Some(UserId(1)));
    println!("{:?}", create_result);
    assert!(create_result.is_ok());

    let companies_packages = create_result.unwrap();
    // read search
    println!("run search companies_packages by id");
    let read_result = core.run(http_client.request_with_auth_header::<CompaniesPackages>(
        Method::Get,
        get_url_request_by_id(base_url.clone(), companies_packages.id),
        None,
        None,
    ));
    println!("{:?}", read_result);
    assert!(read_result.is_err());

    // read companies
    println!("run search companies by package id");
    let read_result = core.run(http_client.request_with_auth_header::<Vec<Company>>(
        Method::Get,
        get_url_request_by_package_id(base_url.clone(), package_id),
        None,
        None,
    ));
    println!("companies by package id {:?}", read_result);
    assert!(read_result.is_ok());

    // read packages
    println!("run search packages by company id");
    let read_result = core.run(http_client.request_with_auth_header::<Vec<Packages>>(
        Method::Get,
        get_url_request_by_company_id(base_url.clone(), company_id),
        None,
        None,
    ));
    println!("packages by company id {:?}", read_result);
    assert!(read_result.is_ok());

    // read available packages
    let country_search = Alpha3("RUS".to_string());
    println!("unauthorized - run search available packages by country {:?}", country_search);
    let read_result = core.run(http_client.request_with_auth_header::<Vec<AvailablePackages>>(
        Method::Get,
        get_url_request_available_packages(base_url.clone(), country_search, 0f64, 0f64),
        None,
        Some(user_id.to_string()),
    ));
    println!("unauthorized - find available packages {:#?}", read_result);
    assert!(read_result.is_ok());

    // delete
    println!("run delete companies_packages ");
    let delete_result = core.run(http_client.request_with_auth_header::<CompaniesPackages>(
        Method::Delete,
        get_url_request_by_id(base_url.clone(), companies_packages.id),
        None,
        None,
    ));
    assert!(delete_result.is_err());

    // delete by super user
    println!("run delete companies_packages by super user ");
    let delete_result = core.run(http_client.request_with_auth_header::<CompaniesPackages>(
        Method::Delete,
        get_url_request_by_id(base_url.clone(), companies_packages.id),
        None,
        Some("1".to_string()),
    ));
    assert!(delete_result.is_ok());

    let user_id = UserId(1);
    // delete
    println!("run delete company ");
    let delete_result = core.run(http_client.request_with_auth_header::<Company>(
        Method::Delete,
        format!("{}/{}/{}", base_url, MOCK_COMPANIES_ENDPOINT, company_id),
        None,
        Some(user_id.to_string()),
    ));
    assert!(delete_result.is_ok());

    // delete
    println!("run delete package ");
    let delete_result = core.run(http_client.request_with_auth_header::<Packages>(
        Method::Delete,
        format!("{}/{}/{}", base_url, MOCK_PACKAGES_ENDPOINT, package_id),
        None,
        Some(user_id.to_string()),
    ));
    assert!(delete_result.is_ok());
}
