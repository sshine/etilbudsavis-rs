//! HTTP client construction and shared request plumbing.

use std::sync::Arc;
use std::time::Duration;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use reqwest::{Method, RequestBuilder};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use url::Url;

use crate::error::{Error, Result, truncate};
use crate::types::{Location, Secret};

/// The production backend the Android app talks to.
pub const DEFAULT_BASE_URL: &str = "https://etilbudsavis.tjek.com";

/// The API key compiled into the Android app. It is public in the APK; the
/// account token, not this key, is the credential.
pub const DEFAULT_API_KEY: &str = "590990e7ad62f206512ce390939ea3d3";

/// `User-Agent` sent unless the builder overrides it.
pub const DEFAULT_USER_AGENT: &str = concat!("etilbudsavis-rs/", env!("CARGO_PKG_VERSION"));

#[derive(Debug)]
pub(crate) struct Inner {
    pub(crate) http: reqwest::Client,
    pub(crate) base: Url,
    pub(crate) api_key: String,
    pub(crate) token: Option<Secret>,
    pub(crate) location: Option<Location>,
}

/// A client for the eTilbudsavis backend.
///
/// Cheap to clone; clones share one connection pool.
#[derive(Debug, Clone)]
pub struct Client {
    pub(crate) inner: Arc<Inner>,
}

impl Client {
    /// Starts building a client.
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    /// A client with default settings and no account token.
    pub fn anonymous() -> Result<Self> {
        Self::builder().build()
    }

    /// A client with a token.
    pub fn with_token(token: impl Into<Secret>) -> Result<Self> {
        Self::builder().token(token).build()
    }

    /// The base URL requests are built against.
    pub fn base_url(&self) -> &Url {
        &self.inner.base
    }

    /// The API key in use.
    pub fn api_key(&self) -> &str {
        &self.inner.api_key
    }

    /// Whether an account token was supplied.
    pub fn has_token(&self) -> bool {
        self.inner.token.is_some()
    }

    /// The location requests carry, when one was supplied.
    pub fn configured_location(&self) -> Option<&Location> {
        self.inner.location.as_ref()
    }

    fn data_url(&self) -> Url {
        self.inner.base.clone()
    }

    fn request(&self, method: Method, url: Url) -> RequestBuilder {
        let mut builder = self
            .inner
            .http
            .request(method, url)
            .header(ACCEPT, "application/json")
            .header("x-api-key", &self.inner.api_key);

        if let Some(token) = &self.inner.token {
            builder = builder.header("x-token", token.expose());
        }
        if let Some(location) = &self.inner.location {
            builder = builder.header(
                "x-location",
                encode_uri_component(&location.header_value().to_string()),
            );
        }
        builder
    }

    /// Reads one data key through the batched NDJSON endpoint.
    ///
    /// `params` is serialized to JSON and paired with `key`, exactly the way the
    /// web app caches and requests data. Use `serde_json::Value::Null` for keys
    /// that take no parameters.
    pub async fn data<T: DeserializeOwned>(&self, key: &str, params: impl Serialize) -> Result<T> {
        let value = self.data_value(key, params).await?;
        serde_json::from_value(value).map_err(|source| Error::Decode {
            expected: std::any::type_name::<T>(),
            body: truncate(&source.to_string(), 512),
            source,
        })
    }

    /// Reads one data key and returns its raw JSON value.
    pub async fn data_value(&self, key: &str, params: impl Serialize) -> Result<Value> {
        let params = canonicalize(serde_json::to_value(params)?);
        let encoded = encode_data_key(key, &params)?;
        let body = json!({ "data": [encoded] });
        let request = self
            .request(Method::POST, self.data_url())
            .header(CONTENT_TYPE, "application/json")
            .json(&body);
        let (status, text) = self.send_text(request).await?;

        if !status.is_success() {
            return Err(api_error(status.as_u16(), &text));
        }

        let record = parse_data_record(&encoded, &text)?;
        match record {
            DataRecord::Success(value) => Ok(value),
            DataRecord::Failure { status, message } => Err(Error::api(status, message)),
        }
    }

    /// Calls one `POST /api/<operation>` command.
    pub async fn call_api<T: DeserializeOwned>(
        &self,
        operation: &str,
        body: impl Serialize,
    ) -> Result<T> {
        let value = self.call_api_value(operation, body).await?;
        serde_json::from_value(value).map_err(|source| Error::Decode {
            expected: std::any::type_name::<T>(),
            body: truncate(&source.to_string(), 512),
            source,
        })
    }

    /// Calls one command and returns its raw JSON value.
    pub async fn call_api_value(&self, operation: &str, body: impl Serialize) -> Result<Value> {
        let url = self.inner.base.join(&format!("api/{operation}"))?;
        let request = self
            .request(Method::POST, url)
            .header(CONTENT_TYPE, "application/json")
            .json(&body);
        let (status, text) = self.send_text(request).await?;

        if !status.is_success() {
            return Err(api_error(status.as_u16(), &text));
        }
        if text.trim().is_empty() {
            return Ok(Value::Null);
        }
        serde_json::from_str(&text).map_err(|source| Error::Decode {
            expected: "a JSON value",
            body: truncate(&text, 512),
            source,
        })
    }

    /// Calls one command that returns `null` on success.
    pub async fn call_api_unit(&self, operation: &str, body: impl Serialize) -> Result<()> {
        self.call_api_value(operation, body).await.map(|_| ())
    }

    async fn send_text(&self, request: RequestBuilder) -> Result<(reqwest::StatusCode, String)> {
        let request = request.build().map_err(Error::Transport)?;
        let method = request.method().clone();
        let path = request.url().path().to_owned();
        let response = self
            .inner
            .http
            .execute(request)
            .await
            .map_err(Error::Transport)?;
        let status = response.status();
        let text = response.text().await.map_err(Error::Transport)?;
        if !status.is_success() {
            tracing::debug!(%status, %method, %path, "request failed");
        }
        Ok((status, text))
    }
}

/// Builds a [`Client`].
#[derive(Debug, Clone)]
pub struct ClientBuilder {
    base_url: String,
    api_key: String,
    token: Option<Secret>,
    location: Option<Location>,
    timeout: Duration,
    user_agent: String,
    http_client: Option<reqwest::Client>,
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.to_owned(),
            api_key: DEFAULT_API_KEY.to_owned(),
            token: None,
            location: None,
            timeout: Duration::from_secs(30),
            user_agent: DEFAULT_USER_AGENT.to_owned(),
            http_client: None,
        }
    }
}

impl ClientBuilder {
    /// Sets the backend base URL, for staging or tests.
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Overrides the Android API key.
    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = api_key.into();
        self
    }

    /// Supplies an account token.
    pub fn token(mut self, token: impl Into<Secret>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Supplies a location for `x-location` and location-aware responses.
    pub fn location(mut self, location: Location) -> Self {
        self.location = Some(location);
        self
    }

    /// Sets the per-request timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Sets the user agent.
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    /// Supplies a pre-built HTTP client, which is also how to add a proxy or
    /// native TLS. The timeout and user agent are then the caller's concern.
    pub fn http_client(mut self, http_client: reqwest::Client) -> Self {
        self.http_client = Some(http_client);
        self
    }

    /// Builds the client.
    pub fn build(self) -> Result<Client> {
        let mut base = Url::parse(&self.base_url)?;
        if !base.path().ends_with('/') {
            let path = format!("{}/", base.path());
            base.set_path(&path);
        }

        let http = match self.http_client {
            Some(http) => http,
            None => {
                #[cfg(feature = "rustls")]
                {
                    // reqwest is built with `rustls-no-provider`; install ring once
                    // for the process. A second install is harmless.
                    let _ = rustls::crypto::ring::default_provider().install_default();
                }
                reqwest::Client::builder()
                    .timeout(self.timeout)
                    .user_agent(self.user_agent)
                    .build()
                    .map_err(Error::Transport)?
            }
        };

        Ok(Client {
            inner: Arc::new(Inner {
                http,
                base,
                api_key: self.api_key,
                token: self.token,
                location: self.location,
            }),
        })
    }
}

enum DataRecord {
    Success(Value),
    Failure { status: u16, message: String },
}

fn parse_data_record(encoded: &str, text: &str) -> Result<DataRecord> {
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let value: Value = serde_json::from_str(line).map_err(|source| Error::Decode {
            expected: "one NDJSON data record",
            body: truncate(line, 512),
            source,
        })?;

        let key = value.get("key").and_then(Value::as_str).unwrap_or_default();
        if key != encoded {
            continue;
        }

        let status = value
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if status == "success" {
            return Ok(DataRecord::Success(
                value.get("value").cloned().unwrap_or(Value::Null),
            ));
        }

        let error = value.get("error").cloned().unwrap_or(Value::Null);
        let status = error
            .get("status")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            .try_into()
            .unwrap_or(0);
        let message = error
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("the service reported an error")
            .to_owned();
        return Ok(DataRecord::Failure { status, message });
    }

    Err(Error::Decode {
        expected: "a data record for the requested key",
        body: truncate(text, 512),
        source: serde_json::Error::io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "no matching data record in the NDJSON response",
        )),
    })
}

fn api_error(status: u16, text: &str) -> Error {
    if let Ok(value) = serde_json::from_str::<Value>(text) {
        let message = value
            .get("message")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .or_else(|| {
                value
                    .get("error")
                    .and_then(Value::as_str)
                    .map(str::to_owned)
            });
        let code = value
            .get("code")
            .and_then(Value::as_u64)
            .and_then(|code| u16::try_from(code).ok())
            .unwrap_or(status);
        if let Some(message) = message {
            return Error::api(code, message);
        }
    }
    Error::api(status, truncate(text, 512))
}

fn encode_data_key(key: &str, params: &Value) -> Result<String> {
    let raw = serde_json::to_vec(&json!([key, params]))?;
    Ok(BASE64.encode(raw))
}

/// Recursively sorts object keys, matching the web app's cache keys.
fn canonicalize(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut entries: Vec<_> = map.into_iter().collect();
            entries.sort_by(|left, right| left.0.cmp(&right.0));
            Value::Object(
                entries
                    .into_iter()
                    .map(|(key, value)| (key, canonicalize(value)))
                    .collect(),
            )
        }
        Value::Array(items) => Value::Array(items.into_iter().map(canonicalize).collect()),
        other => other,
    }
}

/// Percent-encodes the way JavaScript's `encodeURIComponent` does.
fn encode_uri_component(text: &str) -> String {
    let mut encoded = String::with_capacity(text.len());
    for byte in text.bytes() {
        match byte {
            b'A'..=b'Z'
            | b'a'..=b'z'
            | b'0'..=b'9'
            | b'-'
            | b'_'
            | b'.'
            | b'!'
            | b'~'
            | b'*'
            | b'\''
            | b'('
            | b')' => encoded.push(char::from(byte)),
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::*;
    use crate::types::Location;

    #[test]
    fn data_keys_are_base64_json_pairs() {
        let key = encode_data_key("groupedFronts", &Value::Null).expect("encode");
        let decoded = BASE64.decode(key).expect("decode");
        assert_eq!(decoded, br#"["groupedFronts",null]"#);
    }

    #[test]
    fn nested_object_keys_are_sorted() {
        let value = serde_json::json!({ "b": 1, "a": { "d": 2, "c": 3 } });
        let canonical = canonicalize(value);
        assert_eq!(
            canonical,
            serde_json::json!({ "a": { "c": 3, "d": 2 }, "b": 1 })
        );
    }

    #[test]
    fn location_header_is_percent_encoded_json() {
        let location = Location {
            latitude: 55.68,
            longitude: 12.57,
            geohash: "u3buy".to_owned(),
            city: None,
            country: Some("DK".to_owned()),
            mode: Some("fallback".to_owned()),
        };
        let encoded = encode_uri_component(&location.header_value().to_string());
        assert!(encoded.contains("%22country%22"));
        assert!(encoded.contains("%22DK%22"));
        assert!(!encoded.contains(' '));
    }
}
