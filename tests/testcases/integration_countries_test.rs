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
        level: 2,
        parent: Some("XAF".to_string().into()),
        alpha2: Alpha2("GN".to_string()),
        alpha3: Alpha3("GIN".to_string()),
        numeric: 0,
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

#[test]
fn test_country() {
    let (mut core, http_client) = super::common::make_utils();
    let base_url = super::common::setup();

    test_country_superuser_crud(&mut core, &http_client, base_url.clone());
    test_country_regular_user_crud(&mut core, &http_client, base_url.clone());
    test_country_unauthorized(&mut core, &http_client, base_url.clone());
    test_read_alpha2_country_unauthorized(&mut core, &http_client, base_url.clone());
    test_read_alpha3_country_unauthorized(&mut core, &http_client, base_url.clone());
    test_read_numeric_country_unauthorized(&mut core, &http_client, base_url.clone());
}

// test country by superuser
fn test_country_superuser_crud(core: &mut tokio_core::reactor::Core, http_client: &HttpClientHandle, base_url: String) {
    let user_id = UserId(1);
    let mut rng = thread_rng();
    let label = CountryLabel(rng.sample_iter(&Alphanumeric).take(7).collect::<String>());
    let url_crud = format!("{}/{}", base_url, MOCK_COUNTRY_ENDPOINT.to_string());

    // create
    println!("run create new country for label {}", label);
    let create_result = create_country(label.clone(), core, http_client, base_url.clone(), Some(user_id.to_string()));
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
fn test_country_regular_user_crud(core: &mut tokio_core::reactor::Core, http_client: &HttpClientHandle, base_url: String) {
    let mut rng = thread_rng();
    let label = CountryLabel(rng.sample_iter(&Alphanumeric).take(7).collect::<String>());
    let url_crud = format!("{}/{}", base_url, MOCK_COUNTRY_ENDPOINT.to_string());

    // create user for test acl
    let user_id = UserId(2);
    let create_role_result = super::common::create_user_role(user_id.clone(), core, http_client, base_url.clone());
    assert!(create_role_result.is_ok());

    // create
    println!("run create new country for label {} for regular user", label);
    let create_result = create_country(label.clone(), core, http_client, base_url.clone(), Some(user_id.to_string()));
    println!("{:?}", create_result);
    assert!(create_result.is_err());

    // create by super user
    println!("run create new country for label {}", label);
    let create_result = create_country(label.clone(), core, http_client, base_url.clone(), Some(UserId(1).to_string()));
    println!("{:?}", create_result);
    assert!(create_result.is_ok());

    // read
    println!("run read country for label {}", label);
    let read_result =
        core.run(http_client.request_with_auth_header::<Country>(Method::Get, url_crud.clone(), None, Some(user_id.to_string())));
    println!("{:?}", read_result);
    assert!(read_result.is_ok());

    // delete user role
    let delete_result = super::common::delete_role(user_id.clone(), core, http_client, base_url.clone());
    assert!(delete_result.is_ok());
}

// test update country without authorization data
fn test_country_unauthorized(core: &mut tokio_core::reactor::Core, http_client: &HttpClientHandle, base_url: String) {
    let mut rng = thread_rng();
    let label = CountryLabel(rng.sample_iter(&Alphanumeric).take(7).collect::<String>());
    let url_crud = format!("{}/{}", base_url, MOCK_COUNTRY_ENDPOINT.to_string());

    // create
    println!("run create new country for label {}", label);
    let create_result = create_country(label.clone(), core, http_client, base_url.clone(), None);
    println!("{:?}", create_result);
    assert!(create_result.is_err());

    // create by super user
    println!("run create new country for label {}", label);
    let create_result = create_country(label.clone(), core, http_client, base_url.clone(), Some(UserId(1).to_string()));
    println!("{:?}", create_result);
    assert!(create_result.is_ok());

    // read
    println!("run read country for label {}", label);
    let read_result = core.run(http_client.request_with_auth_header::<Country>(Method::Get, url_crud.clone(), None, None));
    println!("{:?}", read_result);
    assert!(read_result.is_ok());
}

fn test_read_alpha2_country_unauthorized(core: &mut tokio_core::reactor::Core, http_client: &HttpClientHandle, base_url: String) {
    let alpha2 = Alpha2("ru".to_string());
    // read
    println!("run read country by  alpha2: {:?}", alpha2);
    let read_result = core.run(http_client.request_with_auth_header::<Country>(
        Method::Get,
        format!("{}/{}/alpha2/{}", base_url, MOCK_COUNTRY_ENDPOINT, alpha2.0),
        None,
        None,
    ));
    println!("{:?}", read_result);
    assert!(read_result.is_ok());
}

fn test_read_alpha3_country_unauthorized(core: &mut tokio_core::reactor::Core, http_client: &HttpClientHandle, base_url: String) {
    let codes = vec![Alpha3("rus".to_string()), Alpha3("XEU".to_string())];

    for item in codes {
        // read
        println!("run read country by  alpha3: {:?}", item);
        let read_result = core.run(http_client.request_with_auth_header::<Country>(
            Method::Get,
            format!("{}/{}/alpha3/{}", base_url, MOCK_COUNTRY_ENDPOINT, item.0),
            None,
            None,
        ));
        println!("{:?}", read_result);
        assert!(read_result.is_ok());
    }
}

fn test_read_numeric_country_unauthorized(core: &mut tokio_core::reactor::Core, http_client: &HttpClientHandle, base_url: String) {
    let numeric = 643;
    // read
    println!("run read country by  numeric: {}", numeric);
    let read_result = core.run(http_client.request_with_auth_header::<Country>(
        Method::Get,
        format!("{}/{}/numeric/{}", base_url, MOCK_COUNTRY_ENDPOINT, numeric),
        None,
        None,
    ));
    println!("{:?}", read_result);
    assert!(read_result.is_ok());
}
