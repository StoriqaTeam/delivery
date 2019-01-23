use hyper::Method;

use lib::models::*;
use stq_static_resources::Currency;
use stq_types::*;

use stq_http::client::{self, ClientHandle as HttpClientHandle};

static MOCK_COMPANIES_ENDPOINT: &'static str = "companies";

fn create_update_company(name: &str) -> UpdateCompany {
    UpdateCompany {
        name: Some(name.to_string()),
        label: None,
        description: None,
        deliveries_from: None,
        logo: None,
        currency: None,
    }
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
        currency: Currency::STQ,
    };

    let body: String = serde_json::to_string(&new_company).unwrap().to_string();
    let create_result = core.run(http_client.request_with_auth_header::<Company>(
        Method::Post,
        get_url_request(base_url),
        Some(body),
        user_id.map(|u| u.to_string()),
    ));

    create_result
}

fn get_url_request_by_id(base_url: String, company_id: CompanyId) -> String {
    format!("{}/{}/{}", base_url, MOCK_COMPANIES_ENDPOINT, company_id)
}

fn get_url_request(base_url: String) -> String {
    format!("{}/{}", base_url, MOCK_COMPANIES_ENDPOINT)
}

#[test]
fn test_company() {
    let (mut core, http_client) = super::common::make_utils();
    let base_url = super::common::setup();

    test_company_superuser_crud(&mut core, &http_client, base_url.clone());
    test_company_regular_user_crud(&mut core, &http_client, base_url.clone());
    test_company_unauthorized(&mut core, &http_client, base_url.clone());
}

// test company by superuser
fn test_company_superuser_crud(core: &mut tokio_core::reactor::Core, http_client: &HttpClientHandle, base_url: String) {
    let user_id = UserId(1);
    let name = "US UPS".to_string();
    // create
    println!("run create new company ");
    let create_result = create_company(name.clone(), core, http_client, base_url.clone(), Some(user_id));
    println!("{:?}", create_result);
    assert!(create_result.is_ok());

    let company = create_result.unwrap();
    // read search
    println!("run search company by id");
    let read_result = core.run(http_client.request_with_auth_header::<Company>(
        Method::Get,
        get_url_request_by_id(base_url.clone(), company.id),
        None,
        Some(user_id.to_string()),
    ));
    println!("{:?}", read_result);
    assert!(read_result.is_ok());

    // update
    println!("run update company ");
    let update_company = create_update_company("UPS USA 2");
    let update_body: String = serde_json::to_string(&update_company).unwrap().to_string();
    let update_result = core.run(http_client.request_with_auth_header::<Company>(
        Method::Put,
        get_url_request_by_id(base_url.clone(), company.id),
        Some(update_body),
        Some(user_id.to_string()),
    ));
    println!("{:?}", update_result);
    assert!(update_result.is_ok());

    // delete
    println!("run delete company ");
    let delete_result = core.run(http_client.request_with_auth_header::<Company>(
        Method::Delete,
        get_url_request_by_id(base_url.clone(), company.id),
        None,
        Some(user_id.to_string()),
    ));
    assert!(delete_result.is_ok());
}

// test company by regular user
fn test_company_regular_user_crud(core: &mut tokio_core::reactor::Core, http_client: &HttpClientHandle, base_url: String) {
    // create user for test acl
    let user_id = UserId(1123);
    let create_role_result = super::common::create_user_role(user_id.clone(), core, http_client, base_url.clone());
    assert!(create_role_result.is_ok());

    let name = "US UPS".to_string();
    // create
    println!("run create new company ");
    let create_result = create_company(name.clone(), core, http_client, base_url.clone(), Some(user_id));
    println!("{:?}", create_result);
    assert!(create_result.is_err());

    // create by super user
    println!("run create new company by super user");
    let create_result = create_company(name.clone(), core, http_client, base_url.clone(), Some(UserId(1)));
    println!("{:?}", create_result);
    assert!(create_result.is_ok());

    let company = create_result.unwrap();
    // read search
    println!("run search company by id");
    let read_result = core.run(http_client.request_with_auth_header::<Company>(
        Method::Get,
        get_url_request_by_id(base_url.clone(), company.id),
        None,
        Some(user_id.to_string()),
    ));
    println!("{:?}", read_result);
    assert!(read_result.is_ok());

    // update
    println!("run update company ");
    let update_company = create_update_company("UPS USA 2");
    let update_body: String = serde_json::to_string(&update_company).unwrap().to_string();
    let update_result = core.run(http_client.request_with_auth_header::<Company>(
        Method::Put,
        get_url_request_by_id(base_url.clone(), company.id),
        Some(update_body),
        Some(user_id.to_string()),
    ));
    println!("{:?}", update_result);
    assert!(update_result.is_err());

    // delete
    println!("run delete company ");
    let delete_result = core.run(http_client.request_with_auth_header::<Company>(
        Method::Delete,
        get_url_request_by_id(base_url.clone(), company.id),
        None,
        Some(user_id.to_string()),
    ));
    assert!(delete_result.is_err());

    // delete by super user
    println!("run delete company by super user ");
    let delete_result = core.run(http_client.request_with_auth_header::<Company>(
        Method::Delete,
        get_url_request_by_id(base_url.clone(), company.id),
        None,
        Some("1".to_string()),
    ));
    assert!(delete_result.is_ok());
}

// test update company without authorization data
fn test_company_unauthorized(core: &mut tokio_core::reactor::Core, http_client: &HttpClientHandle, base_url: String) {
    let name = "US UPS".to_string();

    // create
    println!("run create new company ");
    let create_result = create_company(name.clone(), core, http_client, base_url.clone(), None);
    println!("{:?}", create_result);
    assert!(create_result.is_err());

    // create by super user
    println!("run create new company by super user");
    let create_result = create_company(name.clone(), core, http_client, base_url.clone(), Some(UserId(1)));
    println!("{:?}", create_result);
    assert!(create_result.is_ok());

    let company = create_result.unwrap();
    // read search
    println!("run search company by id");
    let read_result = core.run(http_client.request_with_auth_header::<Company>(
        Method::Get,
        get_url_request_by_id(base_url.clone(), company.id),
        None,
        None,
    ));
    println!("{:?}", read_result);
    assert!(read_result.is_ok());

    // update
    println!("run update company ");
    let update_company = create_update_company("UPS USA 2");
    let update_body: String = serde_json::to_string(&update_company).unwrap().to_string();
    let update_result = core.run(http_client.request_with_auth_header::<Company>(
        Method::Put,
        get_url_request_by_id(base_url.clone(), company.id),
        Some(update_body),
        None,
    ));
    println!("{:?}", update_result);
    assert!(update_result.is_err());

    // delete
    println!("run delete company ");
    let delete_result = core.run(http_client.request_with_auth_header::<Company>(
        Method::Delete,
        get_url_request_by_id(base_url.clone(), company.id),
        None,
        None,
    ));
    assert!(delete_result.is_err());

    // delete by super user
    println!("run delete company by super user ");
    let delete_result = core.run(http_client.request_with_auth_header::<Company>(
        Method::Delete,
        get_url_request_by_id(base_url.clone(), company.id),
        None,
        Some("1".to_string()),
    ));
    assert!(delete_result.is_ok());
}
