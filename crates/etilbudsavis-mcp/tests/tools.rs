//! Calling the tool implementations against a mock backend.
#![allow(clippy::expect_used)]

use base64::Engine as _;
use etilbudsavis_api::Client;
use etilbudsavis_mcp::render::OutputLimits;
use etilbudsavis_mcp::server::{EtibudsavisServer, Mode};
use etilbudsavis_mcp::tools::{BusinessesParams, CatalogueParams};
use rmcp::handler::server::wrapper::Parameters;
use serde_json::json;
use wiremock::matchers::{method, path};
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
        "isStatic": false,
        "isEnabled": true,
        "primaryCategoryId": "groceries",
        "primaryColor": "#01AA46",
        "isWebstore": false
    })
}

#[tokio::test]
async fn businesses_tool_pages_the_rows() {
    let server = MockServer::start().await;
    let body = json!({
        "key": data_key("businesses", json!({ "limit": 1 })),
        "status": "success",
        "value": [business()]
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
    let tool = EtibudsavisServer::new(client, OutputLimits::default(), Mode::default());
    let result = tool
        .businesses(Parameters(BusinessesParams {
            category: None,
            limit: Some(1),
        }))
        .await;
    let body = result.structured_content.expect("structured content");
    assert_eq!(body["total"], json!(1));
    assert_eq!(body["items"][0]["name"], json!("365discount"));
}

#[tokio::test]
async fn catalogue_groups_are_pageable() {
    let server = MockServer::start().await;
    let body = json!({
        "key": data_key("groupedFronts", serde_json::Value::Null),
        "status": "success",
        "value": [[
            "groceries",
            [{
                "category": "groceries",
                "business": business(),
                "publications": [{
                    "id": "pub-1",
                    "label": "Uge 45",
                    "thumb": "https://example.test/thumb.png",
                    "view": "https://example.test/view.png",
                    "validFrom": "2026-11-02T23:00:00.000Z",
                    "validUntil": "2026-11-09T23:00:00.000Z",
                    "displayValidUntil": true,
                    "publish": "2026-11-01T08:00:43.200Z"
                }]
            }]
        ]]
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
    let tool = EtibudsavisServer::new(client, OutputLimits::default(), Mode::default());
    let result = tool
        .catalogue(Parameters(CatalogueParams {
            category: Some("groceries".to_owned()),
            limit: Some(10),
        }))
        .await;
    let body = result.structured_content.expect("structured content");
    assert_eq!(body["total"], json!(1));
    assert_eq!(body["items"][0]["publication"]["id"], json!("pub-1"));
}

#[tokio::test]
async fn a_server_without_a_token_reports_the_session_as_absent() {
    let server = MockServer::start().await;
    let client = Client::builder()
        .base_url(server.uri())
        .build()
        .expect("client");
    let tool = EtibudsavisServer::new(client, OutputLimits::default(), Mode::default());
    let body = tool
        .session_status()
        .await
        .structured_content
        .expect("structured content");
    assert_eq!(body["authenticated"], json!(false));
    assert_eq!(body["token_present"], json!(false));
}
