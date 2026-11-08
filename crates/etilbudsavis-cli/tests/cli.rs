//! The CLI against a mock backend.
#![allow(clippy::expect_used)]

use assert_cmd::Command;
use base64::Engine as _;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn data_key(key: &str, params: serde_json::Value) -> String {
    base64::engine::general_purpose::STANDARD
        .encode(serde_json::to_vec(&json!([key, params])).expect("json"))
}

fn command(server: &MockServer) -> Command {
    let mut command = Command::cargo_bin("etilbudsavis").expect("binary");
    command
        .arg("--base-url")
        .arg(server.uri())
        .arg("--session-file")
        .arg(std::env::temp_dir().join("etilbudsavis-test-token.json"))
        .arg("--json");
    command
}

#[tokio::test]
async fn business_list_prints_the_rows_as_json() {
    let server = MockServer::start().await;
    let body = json!({
        "key": data_key("businesses", json!({ "limit": 1 })),
        "status": "success",
        "value": [{
            "publicId": "DWZE1w",
            "name": "365discount",
            "slugs": ["365discount"],
            "countryCode": "DK",
            "isStatic": false,
            "isEnabled": true,
            "primaryCategoryId": "groceries",
            "primaryColor": "#01AA46",
            "isWebstore": false
        }]
    })
    .to_string();
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let output = command(&server)
        .args(["business", "list", "--limit", "1"])
        .output()
        .expect("run");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(value[0]["name"], json!("365discount"));
}

#[tokio::test]
async fn help_lists_the_domains() {
    let output = Command::cargo_bin("etilbudsavis")
        .expect("binary")
        .arg("--help")
        .output()
        .expect("run");
    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    for domain in [
        "auth",
        "catalogue",
        "business",
        "offer",
        "store",
        "search",
        "lists",
    ] {
        assert!(text.contains(domain), "help is missing {domain}");
    }
}

#[tokio::test]
async fn a_missing_account_is_reported_without_a_token() {
    let server = MockServer::start().await;
    let output = command(&server)
        .args(["auth", "status"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(value["authenticated"], json!(false));
}
