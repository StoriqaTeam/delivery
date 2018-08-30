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

fn get_url_request_by_id(base_url: String, companies_packages_id: CompanyPackageId) -> String {
    format!("{}/{}/{}", base_url, MOCK_COMPANIES_PACKAGES_ENDPOINT, companies_packages_id)
}

fn get_url_request(base_url: String) -> String {
    format!("{}/{}", base_url, MOCK_COMPANIES_PACKAGES_ENDPOINT)
}

// test companies_packages by superuser
#[test]
fn test_companies_packages_superuser_crud() {
    let (mut core, http_client) = common::make_utils();
    let base_url = common::setup();
    let user_id = UserId(1);
    let company_id = CompanyId(1);
    let package_id = PackageId(1);
    // create
    println!("run create new companies_packages ");
    let create_result = create_companies_packages(company_id, package_id, &mut core, &http_client, base_url.clone(), Some(user_id));
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

    // delete
    println!("run delete companies_packages ");
    let delete_result = core.run(http_client.request_with_auth_header::<CompaniesPackages>(
        Method::Delete,
        get_url_request_by_id(base_url.clone(), companies_packages.id),
        None,
        Some(user_id.to_string()),
    ));
    assert!(delete_result.is_ok());
}

// test companies_packages by regular user
#[test]
fn test_companies_packages_regular_user_crud() {
    let (mut core, http_client) = common::make_utils();
    let base_url = common::setup();
    let company_id = CompanyId(1);
    let package_id = PackageId(1);

    // create user for test acl
    let user_id = UserId(1123);
    let create_role_result = common::create_user_role(user_id.clone(), &mut core, &http_client, base_url.clone());
    assert!(create_role_result.is_ok());

    // create
    println!("run create new companies_packages ");
    let create_result = create_companies_packages(company_id, package_id, &mut core, &http_client, base_url.clone(), Some(user_id));
    println!("{:?}", create_result);
    assert!(create_result.is_err());

    // create by super user
    println!("run create new companies_packages by super user");
    let create_result = create_companies_packages(company_id, package_id, &mut core, &http_client, base_url.clone(), Some(UserId(1)));
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
}

// test update companies_packages without authorization data
#[test]
fn test_companies_packages_unauthorized() {
    let (mut core, http_client) = common::make_utils();
    let base_url = common::setup();
    let company_id = CompanyId(1);
    let package_id = PackageId(1);

    // create
    println!("run create new companies_packages ");
    let create_result = create_companies_packages(company_id, package_id, &mut core, &http_client, base_url.clone(), None);
    println!("{:?}", create_result);
    assert!(create_result.is_err());

    // create by super user
    println!("run create new companies_packages by super user");
    let create_result = create_companies_packages(company_id, package_id, &mut core, &http_client, base_url.clone(), Some(UserId(1)));
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
}
