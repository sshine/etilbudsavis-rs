//! Location lookup.

use serde_json::{Value, json};

use crate::client::Client;
use crate::error::Result;
use crate::types::{Location, LocationInfo};

impl Client {
    /// The service's fallback location, or the one the request carried.
    pub async fn location(&self) -> Result<Location> {
        self.data("location", Value::Null).await
    }

    /// Reverse-geocodes a point into a geohash and country.
    pub async fn location_info(&self, latitude: f64, longitude: f64) -> Result<LocationInfo> {
        self.data(
            "locationInfo",
            json!({ "latitude": latitude, "longitude": longitude }),
        )
        .await
    }
}
