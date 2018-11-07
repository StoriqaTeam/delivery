extern crate futures;
extern crate hyper;
extern crate rand;
extern crate serde_json;
extern crate stq_http;
extern crate stq_types;
extern crate tokio_core;

extern crate delivery_lib as lib;

use lib::models::*;
use stq_types::*;

use self::futures::prelude::*;
use self::rand::Rng;
use self::stq_http::client::{self, Client as HttpClient, ClientHandle as HttpClientHandle, Config as HttpConfig};
use self::tokio_core::reactor::Core;
use hyper::Method;
use std::sync::mpsc::channel;
use std::thread;

pub fn setup() -> String {
    let (tx, rx) = channel::<bool>();
    let mut rng = rand::thread_rng();
    let port = rng.gen_range(40000, 60000);
    thread::spawn({
        let tx = tx.clone();
        move || {
            let config = lib::config::Config::new().expect("Can't load app config!");
            lib::start_server(config, Some(port), move || {
                let _ = tx.send(true);
            });
        }
    });
    rx.recv().unwrap();

    format!("http://localhost:{}", port)
}

pub fn make_utils() -> (Core, HttpClientHandle) {
    let core = Core::new().expect("Unexpected error creating event loop core");
    let client = HttpClient::new(
        &HttpConfig {
            http_client_retries: 3,
            http_client_buffer_size: 3,
            timeout_duration_ms: 5000,
        },
        &core.handle(),
    );
    let client_handle = client.handle();
    core.handle().spawn(client.stream().for_each(|_| Ok(())));
    (core, client_handle)
}

pub fn create_user_role(
    user_id: UserId,
    core: &mut tokio_core::reactor::Core,
    http_client: &HttpClientHandle,
    base_url: String,
) -> Result<UserRole, client::Error> {
    let new_role = NewUserRole {
        user_id,
        name: DeliveryRole::User,
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

pub fn create_user_store_role(
    user_id: UserId,
    store_id: StoreId,
    core: &mut tokio_core::reactor::Core,
    http_client: &HttpClientHandle,
    base_url: String,
) -> Result<UserRole, client::Error> {
    let new_role = NewUserRole {
        user_id,
        name: DeliveryRole::StoreManager,
        id: RoleId::new(),
        data: Some(serde_json::to_value(store_id.0).unwrap()),
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

pub fn delete_role(
    user_id: UserId,
    core: &mut tokio_core::reactor::Core,
    http_client: &HttpClientHandle,
    url: String,
) -> Result<Vec<UserRole>, client::Error> {
    let super_user_id = UserId(1);
    core.run(http_client.request_with_auth_header::<Vec<UserRole>>(
        Method::Delete,
        format!("{}/roles/by-user-id/{}", url, user_id.to_string()),
        None,
        Some(super_user_id.to_string()),
    ))
}

static MOCK_COMPANIES_PACKAGES_ENDPOINT: &'static str = "companies_packages";
static MOCK_COMPANIES_ENDPOINT: &'static str = "companies";
static MOCK_PACKAGES_ENDPOINT: &'static str = "packages";

fn create_companies_packages(
    payload: NewCompaniesPackages,
    core: &mut tokio_core::reactor::Core,
    http_client: &HttpClientHandle,
    base_url: String,
    user_id: Option<UserId>,
) -> Result<CompaniesPackages, client::Error> {
    let body: String = serde_json::to_string(&payload).unwrap().to_string();
    let create_result = core.run(http_client.request_with_auth_header::<CompaniesPackages>(
        Method::Post,
        format!("{}/{}", base_url, MOCK_COMPANIES_PACKAGES_ENDPOINT),
        Some(body),
        user_id.map(|u| u.to_string()),
    ));

    create_result
}

fn create_company(
    payload: NewCompany,
    core: &mut tokio_core::reactor::Core,
    http_client: &HttpClientHandle,
    base_url: String,
    user_id: Option<UserId>,
) -> Result<Company, client::Error> {
    let body: String = serde_json::to_string(&payload).unwrap().to_string();
    let create_result = core.run(http_client.request_with_auth_header::<Company>(
        Method::Post,
        format!("{}/{}", base_url, MOCK_COMPANIES_ENDPOINT),
        Some(body),
        user_id.map(|u| u.to_string()),
    ));

    create_result
}

fn create_package(
    payload: NewPackages,
    core: &mut tokio_core::reactor::Core,
    http_client: &HttpClientHandle,
    base_url: String,
    user_id: Option<UserId>,
) -> Result<Packages, client::Error> {
    let body: String = serde_json::to_string(&payload).unwrap().to_string();
    let create_result = core.run(http_client.request_with_auth_header::<Packages>(
        Method::Post,
        format!("{}/{}", base_url, MOCK_PACKAGES_ENDPOINT),
        Some(body),
        user_id.map(|u| u.to_string()),
    ));

    create_result
}

pub fn create_delivery_objects(
    payload: (NewCompany, NewPackages),
    core: &mut tokio_core::reactor::Core,
    http_client: &HttpClientHandle,
    base_url: String,
    user_id: Option<UserId>,
) -> (PackageId, CompanyId, CompanyPackageId) {
    let (new_company, new_package) = payload;
    let create_result = create_package(new_package, core, http_client, base_url.clone(), user_id);
    assert!(create_result.is_ok(), "Can not create package");
    let package_id = create_result.unwrap().id;

    let create_result = create_company(new_company, core, http_client, base_url.clone(), user_id);
    println!("result create company {:#?}", create_result);
    assert!(create_result.is_ok(), "Can not create company");
    let company_id = create_result.unwrap().id;

    let new_company_package = NewCompaniesPackages {
        company_id: company_id.clone(),
        package_id: package_id.clone(),
    };

    let create_result = create_companies_packages(new_company_package, core, http_client, base_url.clone(), user_id);
    assert!(create_result.is_ok(), "Can not create company_package");
    let companies_package_id = create_result.unwrap().id;

    (package_id, company_id, companies_package_id)
}

pub fn delete_deliveries_objects(
    ids: (PackageId, CompanyId, CompanyPackageId),
    core: &mut tokio_core::reactor::Core,
    http_client: &HttpClientHandle,
    base_url: String,
    user_id: UserId,
) {
    let (package_id, company_id, _companies_package_id) = ids;

    let delete_result = core.run(http_client.request_with_auth_header::<CompaniesPackages>(
        Method::Delete,
        format!(
            "{}/{}/{}/{}/{}",
            base_url, MOCK_COMPANIES_ENDPOINT, company_id, MOCK_PACKAGES_ENDPOINT, package_id
        ),
        None,
        Some(user_id.to_string()),
    ));
    assert!(delete_result.is_ok(), "Can not delete company_package");

    let delete_result = core.run(http_client.request_with_auth_header::<Company>(
        Method::Delete,
        format!("{}/{}/{}", base_url, MOCK_COMPANIES_ENDPOINT, company_id),
        None,
        Some(user_id.to_string()),
    ));
    assert!(delete_result.is_ok(), "Can not delete company");

    let delete_result = core.run(http_client.request_with_auth_header::<Packages>(
        Method::Delete,
        format!("{}/{}/{}", base_url, MOCK_PACKAGES_ENDPOINT, package_id),
        None,
        Some(user_id.to_string()),
    ));
    assert!(delete_result.is_ok(), "Can not delete package");
}
