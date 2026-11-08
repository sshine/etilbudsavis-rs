//! Public data types shared by the client, CLI, and MCP server.

use std::collections::BTreeMap;
use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A value that must not appear in logs or `Debug` output.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Secret(String);

impl Secret {
    /// Wraps a secret.
    pub fn new(secret: impl Into<String>) -> Self {
        Self(secret.into())
    }

    /// The underlying value, for the one place that needs to send it.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl<T: Into<String>> From<T> for Secret {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret(<redacted>)")
    }
}

impl fmt::Display for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted>")
    }
}

/// Timestamps in the API come in RFC 3339, with numeric offsets without a colon,
/// and in some publication payloads as JavaScript `Date#toString` output.
pub mod timestamp {
    use chrono::{DateTime, NaiveDateTime, SecondsFormat, Utc};
    use serde::{Deserialize, Deserializer};

    /// Parses every timestamp shape the service has been observed to send.
    pub fn parse(text: &str) -> Option<DateTime<Utc>> {
        if let Ok(value) = DateTime::parse_from_rfc3339(text) {
            return Some(value.with_timezone(&Utc));
        }
        for format in [
            "%Y-%m-%dT%H:%M:%S%.f%z",
            "%Y-%m-%dT%H:%M:%S%z",
            "%Y-%m-%d %H:%M:%S%.f%z",
            "%a %b %d %Y %H:%M:%S GMT%z",
        ] {
            if let Ok(value) = DateTime::parse_from_str(text, format) {
                return Some(value.with_timezone(&Utc));
            }
        }
        for format in ["%Y-%m-%dT%H:%M:%S%.f", "%Y-%m-%dT%H:%M:%S"] {
            if let Ok(value) = NaiveDateTime::parse_from_str(text, format) {
                return Some(value.and_utc());
            }
        }
        None
    }

    /// Deserializes an optional timestamp, tolerating every observed format.
    pub fn option<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let text = Option::<String>::deserialize(deserializer)?;
        Ok(text.as_deref().and_then(parse))
    }

    /// The current time as the service writes it.
    pub fn now() -> String {
        Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
    }
}

/// A geographic point.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Coordinates {
    /// Latitude in degrees.
    pub latitude: f64,
    /// Longitude in degrees.
    pub longitude: f64,
}

/// A location the app resolved, or the service's fallback.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Location {
    /// Latitude in degrees.
    #[serde(default)]
    pub latitude: f64,
    /// Longitude in degrees.
    #[serde(default)]
    pub longitude: f64,
    /// Geohash used for radius queries.
    #[serde(default)]
    pub geohash: String,
    /// City name.
    #[serde(default)]
    pub city: Option<String>,
    /// ISO 3166-1 alpha-2 country code.
    #[serde(default)]
    pub country: Option<String>,
    /// How the location was obtained, such as `fallback` or `manual`.
    #[serde(default)]
    pub mode: Option<String>,
}

impl Location {
    /// The subset the API expects in the `x-location` header.
    pub fn header_value(&self) -> Value {
        serde_json::json!({
            "country": self.country,
            "geohash": self.geohash,
            "source": self.mode,
        })
    }
}

/// The result of reverse-geocoding a point.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocationInfo {
    /// Geohash for the point.
    #[serde(default)]
    pub geohash: String,
    /// ISO 3166-1 alpha-2 country code.
    #[serde(default)]
    pub country: Option<String>,
}

/// A page request. The service defaults to 100 rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pagination {
    /// Maximum rows to return.
    pub limit: u32,
    /// Rows to skip.
    pub offset: u32,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            limit: 100,
            offset: 0,
        }
    }
}

impl Pagination {
    /// A page of `limit` rows starting at zero.
    pub fn first(limit: u32) -> Self {
        Self { limit, offset: 0 }
    }

    /// The next page.
    pub fn next(&self) -> Self {
        Self {
            limit: self.limit,
            offset: self.offset.saturating_add(self.limit),
        }
    }
}

/// Pagination echoed by the service.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaginationInfo {
    /// Maximum rows requested.
    #[serde(default)]
    pub limit: u32,
    /// Rows skipped.
    #[serde(default)]
    pub offset: u32,
}

/// The `metadata` object on most list responses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Metadata {
    /// The pagination that produced the list.
    #[serde(default)]
    pub pagination: Option<PaginationInfo>,
}

/// A list response. Offers call the field `metadata`; stores and searches call it
/// `_metadata`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Paginated<T> {
    /// The rows.
    pub data: Vec<T>,
    /// Pagination information, when the service sends it.
    #[serde(rename = "metadata", alias = "_metadata", default)]
    pub metadata: Option<Metadata>,
}

impl<T> Default for Paginated<T> {
    fn default() -> Self {
        Self {
            data: Vec::new(),
            metadata: None,
        }
    }
}

/// A business category group and the fronts inside it.
pub type GroupedFront = (String, Vec<Front>);

/// A business, as embedded in offers, stores, and its own endpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Business {
    /// Stable public identifier, such as `i7NdvM`.
    pub public_id: String,
    /// Display name.
    pub name: String,
    /// URL slugs, one per locale.
    #[serde(default)]
    pub slugs: Vec<String>,
    /// ISO 3166-1 alpha-2 country code.
    #[serde(default)]
    pub country_code: Option<String>,
    /// Full-colour logo on a light background.
    #[serde(default)]
    pub positive_logo_image: Option<String>,
    /// Full-colour logo on a dark background.
    #[serde(default)]
    pub negative_logo_image: Option<String>,
    /// Compact logo mark on a light background.
    #[serde(default)]
    pub positive_logo_mark: Option<String>,
    /// Compact logo mark on a dark background.
    #[serde(default)]
    pub negative_logo_mark: Option<String>,
    /// Primary catalogue category.
    #[serde(default)]
    pub primary_category_id: Option<String>,
    /// Brand colour as `#rrggbb`.
    #[serde(default)]
    pub primary_color: Option<String>,
    /// Whether the business is a fixed fixture of the service.
    #[serde(default)]
    pub is_static: bool,
    /// Whether the business is currently visible.
    #[serde(default)]
    pub is_enabled: bool,
    /// A short marketing description.
    #[serde(default)]
    pub short_description: Option<String>,
    /// The business's own website.
    #[serde(default)]
    pub website: Option<String>,
    /// Whether the business has an online shop.
    #[serde(default)]
    pub is_webstore: bool,
}

impl Business {
    /// The preferred slug, if any.
    pub fn slug(&self) -> Option<&str> {
        self.slugs.first().map(String::as_str)
    }
}

/// A publication (leaflet) summary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Publication {
    /// Public identifier.
    pub id: String,
    /// Human label, such as `Uge 45`.
    #[serde(default)]
    pub label: Option<String>,
    /// Small cover image.
    #[serde(default)]
    pub thumb: Option<String>,
    /// Large cover image.
    #[serde(default)]
    pub view: Option<String>,
    /// Start of the validity window.
    #[serde(default, deserialize_with = "timestamp::option")]
    pub valid_from: Option<DateTime<Utc>>,
    /// End of the validity window.
    #[serde(default, deserialize_with = "timestamp::option")]
    pub valid_until: Option<DateTime<Utc>>,
    /// Whether the UI should show the end date.
    #[serde(default)]
    pub display_valid_until: bool,
    /// When the publication was published.
    #[serde(default, deserialize_with = "timestamp::option")]
    pub publish: Option<DateTime<Utc>>,
}

/// A business plus its currently relevant publications.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Front {
    /// Category slug.
    #[serde(default)]
    pub category: Option<String>,
    /// The business.
    pub business: Business,
    /// Active and upcoming publications.
    #[serde(default)]
    pub publications: Vec<Publication>,
}

/// The metadata wrapper behind the `publication` data key.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicationWrapper {
    /// The publication metadata.
    pub publication: PublicationMeta,
}

/// Publication metadata as the viewer endpoint reports it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicationMeta {
    /// Viewer type, usually `incito`.
    #[serde(default)]
    pub r#type: Option<String>,
    /// Business name.
    #[serde(default)]
    pub name: Option<String>,
    /// Cover thumbnail.
    #[serde(default)]
    pub thumb: Option<String>,
    /// Brand colour.
    #[serde(default)]
    pub primary_color: Option<String>,
    /// Publication label.
    #[serde(default)]
    pub publication_name: Option<String>,
    /// Public identifier.
    pub public_id: String,
    /// Start of validity, as sent.
    #[serde(default)]
    pub valid_from: Option<String>,
    /// End of validity, as sent.
    #[serde(default)]
    pub valid_until: Option<String>,
}

/// One offer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Offer {
    /// Stable public identifier.
    pub public_id: String,
    /// Product name.
    pub name: String,
    /// Longer description, often with pack size and conditions.
    #[serde(default)]
    pub description: Option<String>,
    /// ISO 4217 currency code.
    #[serde(default)]
    pub currency_code: Option<String>,
    /// Unit price, such as kroner per kilogram.
    #[serde(default)]
    pub unit_price: Option<f64>,
    /// Base unit for `unit_price`, such as `kilogram` or `piece`.
    #[serde(default)]
    pub base_unit: Option<String>,
    /// The price in `currency_code`.
    #[serde(default)]
    pub price: Option<f64>,
    /// Absolute saving.
    #[serde(default)]
    pub savings: Option<f64>,
    /// Lowest price when a range is offered.
    #[serde(default)]
    pub from_price: Option<f64>,
    /// Membership price.
    #[serde(default)]
    pub membership_price: Option<f64>,
    /// App-only price.
    #[serde(default)]
    pub app_price: Option<f64>,
    /// Relative saving as the service computes it.
    #[serde(default)]
    pub relative_savings: Option<f64>,
    /// Relative membership saving.
    #[serde(default)]
    pub membership_relative_savings: Option<f64>,
    /// Free items in a multi-buy.
    #[serde(default)]
    pub piece_count_get: Option<i64>,
    /// Items that must be bought in a multi-buy.
    #[serde(default)]
    pub piece_count_get_for: Option<i64>,
    /// Link to an online shop.
    #[serde(default)]
    pub webshop_link: Option<String>,
    /// Small product image.
    #[serde(default)]
    pub image: Option<String>,
    /// Large product image.
    #[serde(default)]
    pub image_large: Option<String>,
    /// Start of the offer window.
    #[serde(default, deserialize_with = "timestamp::option")]
    pub valid_from: Option<DateTime<Utc>>,
    /// End of the offer window.
    #[serde(default, deserialize_with = "timestamp::option")]
    pub valid_until: Option<DateTime<Utc>>,
    /// When the offer becomes visible.
    #[serde(default, deserialize_with = "timestamp::option")]
    pub visible_from: Option<DateTime<Utc>>,
    /// Whether the UI should show the end date.
    #[serde(default)]
    pub display_valid_until: bool,
    /// The business that published the offer.
    #[serde(default)]
    pub business: Option<Business>,
    /// Convenience copy of the business id.
    #[serde(default)]
    pub business_public_id: Option<String>,
    /// The publication the offer belongs to.
    #[serde(default)]
    pub publication_public_id: Option<String>,
    /// Cover of that publication.
    #[serde(default)]
    pub publication_cover_image: Option<String>,
    /// Display unit for the pack size, such as `g`.
    #[serde(default)]
    pub unit_symbol: Option<String>,
    /// Lower bound of the piece count.
    #[serde(default)]
    pub piece_count_from: Option<i64>,
    /// Upper bound of the piece count.
    #[serde(default)]
    pub piece_count_to: Option<i64>,
    /// Minimum piece count.
    #[serde(default)]
    pub piece_count_min: Option<i64>,
    /// Maximum piece count.
    #[serde(default)]
    pub piece_count_max: Option<i64>,
    /// Lower bound of the unit size.
    #[serde(default)]
    pub unit_size_from: Option<f64>,
    /// Upper bound of the unit size.
    #[serde(default)]
    pub unit_size_to: Option<f64>,
    /// Department slug, used for grouping in shopping lists.
    #[serde(default)]
    pub department_slug: Option<String>,
}

/// A product variant behind an offer image.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfferProduct {
    /// Product image.
    #[serde(default)]
    pub image: Option<String>,
    /// Product name.
    #[serde(default)]
    pub name: Option<String>,
    /// Unit symbol.
    #[serde(default)]
    pub unit_symbol: Option<String>,
    /// Unit size.
    #[serde(default)]
    pub unit_size: Option<f64>,
}

/// Offers grouped by department.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DepartmentOffers {
    /// Department slug.
    pub department_slug: String,
    /// Offers in the department.
    #[serde(default)]
    pub offers: Vec<Offer>,
}

/// Opening hours for one weekday.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpeningHours {
    /// Opening time, `HH:MM:SS`.
    #[serde(default)]
    pub opens: Option<String>,
    /// Closing time, `HH:MM:SS`.
    #[serde(default)]
    pub closes: Option<String>,
    /// Weekday name.
    #[serde(default)]
    pub day_of_week: Option<String>,
}

/// A physical store.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Store {
    /// Store identifier.
    pub id: String,
    /// Store name.
    pub name: String,
    /// Opening hours.
    #[serde(default)]
    pub opening_hours: Vec<OpeningHours>,
    /// Postal code.
    #[serde(default)]
    pub postal_code: Option<String>,
    /// Country code.
    #[serde(default)]
    pub country_code: Option<String>,
    /// City.
    #[serde(default)]
    pub city: Option<String>,
    /// Street address.
    #[serde(default)]
    pub street_address: Option<String>,
    /// Coordinates.
    #[serde(default)]
    pub coordinates: Option<Coordinates>,
    /// Store website.
    #[serde(default)]
    pub website: Option<String>,
    /// The chain the store belongs to.
    #[serde(default)]
    pub business: Option<Business>,
}

/// A popular search term.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TopSearch {
    /// The term.
    #[serde(default)]
    pub title: String,
    /// The result kind.
    #[serde(rename = "type", default)]
    pub kind: Option<String>,
    /// Where the term came from.
    #[serde(default)]
    pub source: Option<String>,
}

/// A search suggestion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    /// The suggestion text.
    #[serde(default)]
    pub title: String,
    /// Optional subtitle.
    #[serde(default)]
    pub subtitle: Option<String>,
    /// The kind of result.
    #[serde(rename = "type", default)]
    pub kind: Option<String>,
    /// A public identifier, when the result names an entity.
    #[serde(default)]
    pub public_id: Option<String>,
    /// Business slugs for a business result.
    #[serde(default)]
    pub market_slugs: Option<Vec<String>>,
    /// Country code for the result.
    #[serde(default)]
    pub market_country_code: Option<String>,
    /// Where the result came from.
    #[serde(default)]
    pub source: Option<String>,
    /// Logo on a light background.
    #[serde(default)]
    pub positive_image: Option<String>,
    /// Logo on a dark background.
    #[serde(default)]
    pub negative_image: Option<String>,
}

/// A search response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchResults {
    /// The suggestions.
    #[serde(default)]
    pub data: Vec<SearchResult>,
    /// Result count metadata.
    #[serde(rename = "_metadata", alias = "metadata", default)]
    pub metadata: Option<SearchMetadata>,
}

/// Size information on a search response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchMetadata {
    /// Number of results.
    #[serde(default)]
    pub size: Option<usize>,
}

/// The SEO landing-page text for a search term.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchSeo {
    /// The payload.
    pub data: SearchSeoData,
}

/// The SEO landing-page content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchSeoData {
    /// Breadcrumb trail.
    #[serde(default)]
    pub breadcrumb: Vec<String>,
    /// Parent categories.
    #[serde(default)]
    pub parent_categories: Vec<String>,
    /// Adjacent categories.
    #[serde(default)]
    pub adjacent_categories: Vec<String>,
    /// Child categories.
    #[serde(default)]
    pub child_categories: Vec<String>,
    /// Markdown description of the term.
    #[serde(default)]
    pub markdown_description: Option<String>,
}

/// A video author.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideoAuthor {
    /// Display name.
    #[serde(default)]
    pub name: Option<String>,
    /// Avatar image.
    #[serde(default)]
    pub image: Option<String>,
    /// Short biography.
    #[serde(default)]
    pub bio: Option<String>,
}

/// A short recipe or shopping-inspiration video.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Video {
    /// Public identifier.
    pub id: String,
    /// Creation timestamp.
    #[serde(default, deserialize_with = "timestamp::option")]
    pub created_at: Option<DateTime<Utc>>,
    /// Video kind.
    #[serde(rename = "type", default)]
    pub kind: Option<String>,
    /// Title.
    #[serde(default)]
    pub name: String,
    /// Description.
    #[serde(default)]
    pub description: Option<String>,
    /// 1080p MP4 URL.
    #[serde(default)]
    pub video1080p: Option<String>,
    /// 720p MP4 URL.
    #[serde(default)]
    pub video720p: Option<String>,
    /// Full-size poster.
    #[serde(default)]
    pub poster: Option<String>,
    /// Poster thumbnail.
    #[serde(default)]
    pub poster_thumb: Option<String>,
    /// The author.
    #[serde(default)]
    pub author: Option<VideoAuthor>,
    /// Country code.
    #[serde(default)]
    pub country_code: Option<String>,
    /// Recipe metadata, kept as sent.
    #[serde(default)]
    pub recipe: Option<Value>,
    /// Whether the current account liked the video.
    #[serde(default)]
    pub liked_by_user: bool,
}

/// A response with a single video.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideoResponse {
    /// The video.
    pub data: Video,
}

/// A response with videos.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideosResponse {
    /// The videos.
    #[serde(default)]
    pub data: Vec<Video>,
}

/// A response with recipe ingredients.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IngredientsResponse {
    /// Ingredients, kept as sent because the shape varies by recipe.
    #[serde(default)]
    pub data: Vec<Value>,
}

/// A person known to the service.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Person {
    /// Internal identifier.
    pub id: String,
    /// Whether the account was created without an email.
    #[serde(default)]
    pub is_anonymous: bool,
    /// Display name.
    #[serde(default)]
    pub name: Option<String>,
    /// Email address.
    #[serde(default)]
    pub email: Option<String>,
    /// Whether the email was verified.
    #[serde(default)]
    pub is_email_verified: bool,
    /// Avatar image.
    #[serde(default)]
    pub avatar_image: Option<String>,
    /// Biography.
    #[serde(default)]
    pub bio: Option<String>,
    /// Feature flags.
    #[serde(default)]
    pub flags: Vec<String>,
    /// Country code.
    #[serde(default)]
    pub country_code: Option<String>,
    /// Privacy-policy consent, when given.
    #[serde(default)]
    pub privacy_policy_consented: Option<bool>,
    /// Enabled notification kinds.
    #[serde(default)]
    pub enabled_notifications: Vec<String>,
}

/// The `auth` data key.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthState {
    /// The signed-in person, or `None` for no session.
    #[serde(default)]
    pub person: Option<Person>,
}

/// The result of creating an anonymous person or verifying an OTP.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthSession {
    /// The person, when the response included it.
    #[serde(default)]
    pub person: Option<Person>,
    /// The bearer token to send as `x-token`.
    pub token: Secret,
    /// The verified email, when the session came from an OTP.
    #[serde(default)]
    pub email: Option<String>,
}

/// The result of starting an email OTP.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OtpStart {
    /// Identifier to pass to `auth-otp-finalize`.
    pub otp_id: String,
    /// Email the code was sent to.
    #[serde(default)]
    pub email: Option<String>,
    /// Expiry, when the service sends one.
    #[serde(default, deserialize_with = "timestamp::option")]
    pub expires_at: Option<DateTime<Utc>>,
}

/// The fields `auth-update-person` can change.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonUpdate {
    /// Country code derived from the location.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_code: Option<String>,
    /// Privacy-policy consent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub privacy_policy_consented: Option<bool>,
    /// Notification kinds to enable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled_notifications: Option<Vec<String>>,
}

/// A shopping list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ShoppingList {
    /// Numeric identifier.
    pub id: i64,
    /// List name.
    pub name: String,
    /// Creation timestamp.
    #[serde(default, deserialize_with = "timestamp::option")]
    pub created_at: Option<DateTime<Utc>>,
    /// Last-change timestamp.
    #[serde(default, deserialize_with = "timestamp::option")]
    pub updated_at: Option<DateTime<Utc>>,
    /// Token used to invite collaborators.
    #[serde(default)]
    pub share_token: Option<String>,
    /// Whether the current account owns the list.
    #[serde(default)]
    pub owned_by_you: bool,
    /// Number of other people on the list.
    #[serde(default)]
    pub collaborator_count: u32,
}

/// One item on a shopping list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShoppingListItem {
    /// Server identifier, absent until the item is saved.
    #[serde(default)]
    pub id: Option<i64>,
    /// Business the item came from.
    #[serde(default)]
    pub business_public_id: Option<String>,
    /// Client-generated identifier, stable across edits.
    pub client_id: String,
    /// Identifier before a merge or split.
    #[serde(default)]
    pub previous_client_id: Option<String>,
    /// Creation timestamp.
    #[serde(default, deserialize_with = "timestamp::option")]
    pub created_at: Option<DateTime<Utc>>,
    /// Last-change timestamp.
    #[serde(default, deserialize_with = "timestamp::option")]
    pub updated_at: Option<DateTime<Utc>>,
    /// Item name.
    pub name: String,
    /// Comment or pack description.
    #[serde(default)]
    pub short_description: Option<String>,
    /// Whether the item is ticked off.
    #[serde(default)]
    pub ticked: bool,
    /// Quantity.
    #[serde(default)]
    pub count: i64,
    /// Estimated price.
    #[serde(default)]
    pub estimated_price: Option<f64>,
    /// Whether the item is still active.
    #[serde(default)]
    pub is_active: bool,
    /// Department for grouping.
    #[serde(default)]
    pub department_slug: Option<String>,
    /// The offer the item was added from, when any.
    #[serde(default)]
    pub offer: Option<Offer>,
}

impl ShoppingListItem {
    /// Creates an unsaved item with a fresh client id and timestamp.
    pub fn draft(name: impl Into<String>) -> ShoppingListItemDraft {
        ShoppingListItemDraft {
            client_id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            count: 1,
            ticked: false,
            created_at: timestamp::now(),
            comment: None,
            estimated_price: None,
            business_public_id: None,
            offer: None,
        }
    }
}

/// A new item to add to a shopping list.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShoppingListItemDraft {
    /// Client-generated identifier.
    pub client_id: String,
    /// Item name.
    pub name: String,
    /// Quantity.
    pub count: i64,
    /// Whether it is already ticked.
    pub ticked: bool,
    /// Creation timestamp.
    pub created_at: String,
    /// Comment or pack description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    /// Estimated price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_price: Option<f64>,
    /// Business the item came from.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub business_public_id: Option<String>,
    /// The offer the item came from.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offer: Option<Offer>,
}

impl ShoppingListItemDraft {
    /// Sets the quantity.
    pub fn count(mut self, count: i64) -> Self {
        self.count = count;
        self
    }

    /// Sets a comment.
    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }

    /// Sets the estimated price.
    pub fn estimated_price(mut self, price: f64) -> Self {
        self.estimated_price = Some(price);
        self
    }
}

/// An edit to a saved shopping-list item.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShoppingListItemUpdate {
    /// Client-generated identifier.
    pub client_id: String,
    /// Item name.
    pub name: String,
    /// Quantity.
    pub count: i64,
    /// Whether it is ticked.
    pub ticked: bool,
    /// Last-change timestamp.
    pub updated_at: String,
    /// Comment or pack description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_description: Option<String>,
    /// Previous client id, for merges.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_client_id: Option<String>,
    /// Business the item came from.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub business_public_id: Option<String>,
    /// Estimated price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_price: Option<f64>,
    /// Department for grouping.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub department_slug: Option<String>,
}

impl ShoppingListItemUpdate {
    /// Builds an update from an existing item.
    pub fn from_item(item: &ShoppingListItem) -> Self {
        Self {
            client_id: item.client_id.clone(),
            name: item.name.clone(),
            count: item.count,
            ticked: item.ticked,
            updated_at: timestamp::now(),
            short_description: item.short_description.clone(),
            previous_client_id: item.previous_client_id.clone(),
            business_public_id: item.business_public_id.clone(),
            estimated_price: item.estimated_price,
            department_slug: item.department_slug.clone(),
        }
    }
}

/// A saved search that watches for new offers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfferAgent {
    /// Public identifier.
    pub public_id: String,
    /// Search term.
    #[serde(default)]
    pub term: String,
    /// Geohash the search is bound to.
    #[serde(default)]
    pub geohash: Option<String>,
    /// Sort order.
    #[serde(default)]
    pub sorting: Option<String>,
    /// Maximum radius in metres.
    #[serde(default)]
    pub max_radius: Option<u64>,
    /// Whether upcoming offers are included.
    #[serde(default)]
    pub include_upcoming: bool,
    /// Minimum price.
    #[serde(default)]
    pub price_min: Option<f64>,
    /// Maximum price.
    #[serde(default)]
    pub price_max: Option<f64>,
    /// Businesses the search is limited to.
    #[serde(default)]
    pub business_ids: Vec<String>,
    /// Whether email notifications are enabled.
    #[serde(default)]
    pub is_email_notifications_enabled: bool,
    /// Whether push notifications are enabled.
    #[serde(default)]
    pub is_push_notifications_enabled: bool,
    /// Last time the agent was viewed.
    #[serde(default, deserialize_with = "timestamp::option")]
    pub last_viewed_at: Option<DateTime<Utc>>,
}

/// The request behind `offer-agent-create`.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewOfferAgent {
    /// Search term.
    pub term: String,
    /// Geohash the search is bound to.
    pub geohash: String,
    /// Sort order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sorting: Option<String>,
    /// Maximum radius in metres.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_radius: Option<u64>,
    /// Whether upcoming offers are included.
    pub include_upcoming: bool,
    /// Minimum price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_min: Option<f64>,
    /// Maximum price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_max: Option<f64>,
    /// Businesses the search is limited to.
    pub business_ids: Vec<String>,
    /// Whether email notifications are enabled.
    pub is_email_notifications_enabled: bool,
    /// Whether push notifications are enabled.
    pub is_push_notifications_enabled: bool,
}

/// The personalised "for you" row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForYou {
    /// Row name.
    #[serde(default)]
    pub name: String,
    /// Offers in the row.
    #[serde(default)]
    pub offers: Vec<Offer>,
}

/// A request body for the `fronts` data key.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrontsQuery {
    /// Restrict to these businesses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub business_ids: Option<Vec<String>>,
    /// Restrict to these business slugs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub business_slugs: Option<Vec<String>>,
    /// Restrict to these local businesses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_business_ids: Option<Vec<String>>,
    /// Coordinates for distance sorting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coordinates: Option<Coordinates>,
    /// Hide businesses without an active publication.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hide_inactive: Option<bool>,
    /// Hide disabled businesses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hide_disabled: Option<bool>,
    /// Restrict to one category.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Keep static fronts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_static: Option<bool>,
    /// Business sort order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub business_sort: Option<Vec<String>>,
    /// Location identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location_id: Option<String>,
    /// Pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<Pagination>,
}

/// A request body for the `businesses` data key.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BusinessesQuery {
    /// Restrict to these businesses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub business_ids: Option<Vec<String>>,
    /// Restrict to one category.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Maximum rows.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

/// A request body for the `stores` data key.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoresQuery {
    /// Restrict to stores carrying this publication.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_id: Option<String>,
    /// Restrict to stores carrying this offer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offer_id: Option<String>,
    /// Restrict to one business.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub business_id: Option<String>,
    /// Restrict to these businesses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub business_ids: Option<Vec<String>>,
    /// Free-text store search.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_term: Option<String>,
    /// Maximum distance in metres.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_radius: Option<u64>,
    /// Pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<Pagination>,
    /// Location identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location_id: Option<String>,
}

impl StoresQuery {
    /// A page of stores.
    pub fn page(limit: u32) -> Self {
        Self {
            pagination: Some(Pagination::first(limit)),
            ..Self::default()
        }
    }
}

/// A request body for the `offers` and `departmentOffers` data keys.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OffersQuery {
    /// Restrict to these businesses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub business_ids: Option<Vec<String>>,
    /// Restrict to this department.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub department_slug: Option<String>,
    /// Restrict to these sources, typically `publication` and `business_product`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sources: Option<Vec<String>>,
    /// Pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<Pagination>,
    /// Sort order, such as `popularity_desc`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<Vec<String>>,
    /// Free-text product search.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_term: Option<String>,
    /// Restrict to these offer ids.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offer_ids: Option<Vec<String>>,
    /// Restrict to these publications.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_ids: Option<Vec<String>>,
    /// Restrict to these stores.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store_ids: Option<Vec<String>>,
    /// Restrict to these category ids.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_ids: Option<Vec<String>>,
    /// Hide offers that have not started yet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hide_upcoming: Option<bool>,
    /// Minimum price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_price: Option<f64>,
    /// Maximum price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_price: Option<f64>,
    /// Maximum distance in metres.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_radius: Option<u64>,
    /// Coordinates for distance filtering.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coordinates: Option<Coordinates>,
    /// Location identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location_id: Option<String>,
}

impl OffersQuery {
    /// A page of offers from publications and business products.
    pub fn page(limit: u32) -> Self {
        Self {
            sources: Some(vec![
                "publication".to_owned(),
                "business_product".to_owned(),
            ]),
            pagination: Some(Pagination::first(limit)),
            ..Self::default()
        }
    }
}

/// A request body for `relatedOffers`.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelatedOffersQuery {
    /// The offer to find relatives for.
    pub public_id: String,
    /// Sources to search.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sources: Option<Vec<String>>,
}

/// A request body for `offer-agent-create`.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OfferAgentInput {
    /// The search term.
    pub term: String,
    /// Geohash.
    pub geohash: String,
    /// Sort order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sorting: Option<String>,
    /// Maximum radius in metres.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_radius: Option<u64>,
    /// Include upcoming offers.
    pub include_upcoming: bool,
    /// Minimum price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_min: Option<f64>,
    /// Maximum price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_max: Option<f64>,
    /// Businesses to search.
    pub business_ids: Vec<String>,
    /// Email notifications.
    pub is_email_notifications_enabled: bool,
    /// Push notifications.
    pub is_push_notifications_enabled: bool,
}

/// A single business lookup result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchBusiness {
    /// The matching business, if any.
    #[serde(default)]
    pub business: Option<Business>,
}

/// The identifier returned when creating a shopping list or accepting an invite.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreatedShoppingList {
    /// The numeric list identifier.
    pub id: i64,
}

/// A public identifier returned after creating an offer agent or another entity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatedPublicId {
    /// The new entity's public identifier.
    pub public_id: String,
}

/// The department the service chose for a free-text shopping-list item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategorizedItem {
    /// The client id that was categorized.
    pub client_id: String,
    /// The chosen department, when the service found one.
    #[serde(default)]
    pub department_slug: Option<String>,
}

/// Free-form JSON for responses whose shape is not needed by every caller.
pub type JsonObject = BTreeMap<String, Value>;
