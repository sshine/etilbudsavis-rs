//! HTTP behaviour against a mock backend.
#![allow(clippy::expect_used)]

use base64::Engine as _;
use etilbudsavis_api::{AuthSession, BusinessesQuery, Client, Error, Location, OffersQuery};
use serde_json::json;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn data_key(key: &str, params: serde_json::Value) -> String {
    base64::engine::general_purpose::STANDARD
        .encode(serde_json::to_vec(&json!([key, params])).expect("json"))
}

fn business() -> serde_json::Value {
    json!({
        "publicId": "DWZE1w",
        "name": "365discount",
        "slugs": ["365discount"],
        "countryCode": "DK",
        "positiveLogoImage": "https://example.test/positive.png",
        "negativeLogoImage": "https://example.test/negative.png",
        "positiveLogoMark": "https://example.test/mark.png",
        "negativeLogoMark": "https://example.test/mark-dark.png",
        "primaryCategoryId": "groceries",
        "primaryColor": "#01AA46",
        "isStatic": false,
        "isEnabled": true,
        "website": "https://example.test/",
        "isWebstore": false
    })
}

#[tokio::test]
async fn a_data_read_sends_the_android_key_and_decodes_ndjson() {
    let server = MockServer::start().await;
    let key = data_key("businesses", json!({ "category": "groceries" }));
    let body = json!({ "key": key, "status": "success", "value": [business()] }).to_string();
    Mock::given(method("POST"))
        .and(path("/"))
        .and(header("x-api-key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_string(format!("{body}\n")))
        .expect(1)
        .mount(&server)
        .await;

    let client = Client::builder()
        .base_url(server.uri())
        .api_key("test-key")
        .build()
        .expect("client");
    let businesses = client
        .businesses(&BusinessesQuery {
            category: Some("groceries".to_owned()),
            ..BusinessesQuery::default()
        })
        .await
        .expect("businesses");

    assert_eq!(businesses.len(), 1);
    assert_eq!(businesses[0].public_id, "DWZE1w");
    assert_eq!(businesses[0].name, "365discount");
    assert_eq!(businesses[0].slug(), Some("365discount"));
}

#[tokio::test]
async fn a_data_error_record_becomes_an_api_error() {
    let server = MockServer::start().await;
    let key = data_key("groupedFronts", serde_json::Value::Null);
    let body = json!({
        "key": key,
        "status": "error",
        "error": { "status": 401, "message": "Not authenticated" }
    })
    .to_string();
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let client = Client::builder()
        .base_url(server.uri())
        .build()
        .expect("client");
    let error = client
        .grouped_fronts()
        .await
        .expect_err("the record is an error");
    match error {
        Error::Api { status, message } => {
            assert_eq!(status, 401);
            assert_eq!(message, "Not authenticated");
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[tokio::test]
async fn a_command_sends_the_token_and_decodes_the_session() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/auth-create-person"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-token", "account-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "person": {
                "id": "1000001",
                "is_anonymous": true,
                "name": null,
                "email": null,
                "is_email_verified": false,
                "avatar_image": null,
                "bio": "",
                "flags": [],
                "country_code": "DK",
                "privacy_policy_consented": null,
                "enabled_notifications": []
            },
            "token": "new-token"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = Client::builder()
        .base_url(server.uri())
        .api_key("test-key")
        .token("account-token")
        .build()
        .expect("client");
    let session: AuthSession = client.create_anonymous().await.expect("session");
    assert_eq!(session.token.expose(), "new-token");
    assert_eq!(
        session.person.as_ref().map(|person| person.id.as_str()),
        Some("1000001")
    );
}

#[tokio::test]
async fn an_http_error_uses_the_message_the_service_sends() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/auth-create-person"))
        .respond_with(
            ResponseTemplate::new(422)
                .set_body_json(json!({ "code": 422, "message": "Invalid account request" })),
        )
        .mount(&server)
        .await;

    let client = Client::builder()
        .base_url(server.uri())
        .build()
        .expect("client");
    let error = client.create_anonymous().await.expect_err("422");
    match error {
        Error::Api { status, message } => {
            assert_eq!(status, 422);
            assert_eq!(message, "Invalid account request");
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[tokio::test]
async fn offers_serialize_the_web_app_query_shape() {
    let server = MockServer::start().await;
    let key = data_key(
        "offers",
        json!({
            "businessIds": ["DWZE1w"],
            "pagination": { "limit": 5, "offset": 0 },
            "searchTerm": "smør",
            "sources": ["publication", "business_product"]
        }),
    );
    let body = json!({
        "key": key,
        "status": "success",
        "value": {
            "data": [],
            "metadata": { "pagination": { "limit": 5, "offset": 0 } }
        }
    })
    .to_string();
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let client = Client::builder()
        .base_url(server.uri())
        .build()
        .expect("client");
    let page = client
        .offers(&OffersQuery {
            business_ids: Some(vec!["DWZE1w".to_owned()]),
            search_term: Some("smør".to_owned()),
            ..OffersQuery::page(5)
        })
        .await
        .expect("offers");
    assert!(page.data.is_empty());
    assert_eq!(
        page.metadata
            .and_then(|metadata| metadata.pagination)
            .map(|pagination| pagination.limit),
        Some(5)
    );
}

#[tokio::test]
async fn a_configured_location_is_percent_encoded_in_the_header() {
    let server = MockServer::start().await;
    let key = data_key("location", serde_json::Value::Null);
    let body = json!({
        "key": key,
        "status": "success",
        "value": {
            "latitude": 55.68,
            "longitude": 12.57,
            "geohash": "u3buy",
            "city": "København",
            "country": "DK",
            "mode": "manual"
        }
    })
    .to_string();
    Mock::given(method("POST"))
        .and(path("/"))
        .and(header(
            "x-location",
            "%7B%22country%22%3A%22DK%22%2C%22geohash%22%3A%22u3buy%22%2C%22source%22%3A%22manual%22%7D",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let client = Client::builder()
        .base_url(server.uri())
        .location(Location {
            latitude: 55.68,
            longitude: 12.57,
            geohash: "u3buy".to_owned(),
            city: Some("København".to_owned()),
            country: Some("DK".to_owned()),
            mode: Some("manual".to_owned()),
        })
        .build()
        .expect("client");
    let location = client.location().await.expect("location");
    assert_eq!(location.geohash, "u3buy");
}

#[tokio::test]
async fn a_base_url_without_a_trailing_slash_still_reaches_api_routes() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/auth-create-person"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "person": {
                "id": "1",
                "is_anonymous": true,
                "flags": [],
                "enabled_notifications": []
            },
            "token": "t"
        })))
        .mount(&server)
        .await;

    let client = Client::builder()
        .base_url(server.uri())
        .build()
        .expect("client");
    assert_eq!(client.base_url().path(), "/");
    client.create_anonymous().await.expect("session");
}
