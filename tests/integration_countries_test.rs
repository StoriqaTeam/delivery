extern crate delivery_lib as lib;
extern crate futures;
extern crate hyper;
extern crate rand;
extern crate serde_json;
extern crate stq_http;
extern crate stq_static_resources;
extern crate stq_types;
extern crate tokio_core;

pub mod common;

use hyper::Method;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

use stq_http::client::{self, ClientHandle as HttpClientHandle};
use stq_types::*;

use lib::models::*;

static MOCK_COUNTRY_ENDPOINT: &'static str = "countries";

// super user
fn create_country(
    label: CountryLabel,
    core: &mut tokio_core::reactor::Core,
    http_client: &HttpClientHandle,
    base_url: String,
    user_id: Option<String>,
) -> Result<Country, client::Error> {
    let new_country = NewCountry {
        label,
        name: serde_json::from_str("[{\"lang\" : \"en\", \"text\" : \"root\"}]").unwrap(),
        level: 3,
        parent_label: Some("EEE".to_string().into()),
    };

    let body: String = serde_json::to_string(&new_country).unwrap().to_string();
    let create_result = core.run(http_client.request_with_auth_header::<Country>(
        Method::Post,
        format!("{}/{}", base_url, MOCK_COUNTRY_ENDPOINT.to_string()),
        Some(body),
        user_id,
    ));

    create_result
}

// test country by superuser
#[test]
fn test_country_superuser_crud() {
    let (mut core, http_client) = common::make_utils();
    let base_url = common::setup();
    let user_id = UserId(1);
    let mut rng = thread_rng();
    let label = CountryLabel(rng.sample_iter(&Alphanumeric).take(7).collect::<String>());
    let url_crud = format!("{}/{}", base_url, MOCK_COUNTRY_ENDPOINT.to_string());

    // create
    println!("run create new country for label {}", label);
    let create_result = create_country(label.clone(), &mut core, &http_client, base_url.clone(), Some(user_id.to_string()));
    println!("{:?}", create_result);
    assert!(create_result.is_ok());

    // read
    println!("run read country for label {}", label);
    let read_result =
        core.run(http_client.request_with_auth_header::<Country>(Method::Get, url_crud.clone(), None, Some(user_id.to_string())));
    println!("{:?}", read_result);
    assert!(read_result.is_ok());
}

// test country by regular user
#[test]
fn test_country_regular_user_crud() {
    let (mut core, http_client) = common::make_utils();
    let base_url = common::setup();
    let mut rng = thread_rng();
    let label = CountryLabel(rng.sample_iter(&Alphanumeric).take(7).collect::<String>());
    let url_crud = format!("{}/{}", base_url, MOCK_COUNTRY_ENDPOINT.to_string());

    // create user for test acl
    let user_id = UserId(2);
    let create_role_result = common::create_user_role(user_id.clone(), &mut core, &http_client, base_url.clone());
    assert!(create_role_result.is_ok());

    // create
    println!("run create new country for label {} for regular user", label);
    let create_result = create_country(label.clone(), &mut core, &http_client, base_url.clone(), Some(user_id.to_string()));
    println!("{:?}", create_result);
    assert!(create_result.is_err());

    // create by super user
    println!("run create new country for label {}", label);
    let create_result = create_country(
        label.clone(),
        &mut core,
        &http_client,
        base_url.clone(),
        Some(UserId(1).to_string()),
    );
    println!("{:?}", create_result);
    assert!(create_result.is_ok());

    // read
    println!("run read country for label {}", label);
    let read_result =
        core.run(http_client.request_with_auth_header::<Country>(Method::Get, url_crud.clone(), None, Some(user_id.to_string())));
    println!("{:?}", read_result);
    assert!(read_result.is_ok());

    // delete user role
    let delete_result = common::delete_role(user_id.clone(), &mut core, &http_client, base_url.clone());
    assert!(delete_result.is_ok());
}

// test update country without authorization data
#[test]
fn test_country_unauthorized() {
    let (mut core, http_client) = common::make_utils();
    let base_url = common::setup();
    let mut rng = thread_rng();
    let label = CountryLabel(rng.sample_iter(&Alphanumeric).take(7).collect::<String>());
    let url_crud = format!("{}/{}", base_url, MOCK_COUNTRY_ENDPOINT.to_string());

    // create
    println!("run create new country for label {}", label);
    let create_result = create_country(label.clone(), &mut core, &http_client, base_url.clone(), None);
    println!("{:?}", create_result);
    assert!(create_result.is_err());

    // create by super user
    println!("run create new country for label {}", label);
    let create_result = create_country(
        label.clone(),
        &mut core,
        &http_client,
        base_url.clone(),
        Some(UserId(1).to_string()),
    );
    println!("{:?}", create_result);
    assert!(create_result.is_ok());

    // read
    println!("run read country for label {}", label);
    let read_result = core.run(http_client.request_with_auth_header::<Country>(Method::Get, url_crud.clone(), None, None));
    println!("{:?}", read_result);
    assert!(read_result.is_err());
}
