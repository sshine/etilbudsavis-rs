//! Favourite businesses and saved offer searches.

use serde_json::{Value, json};

use crate::client::Client;
use crate::error::Result;
use crate::types::{Business, CreatedPublicId, NewOfferAgent, OfferAgent};

impl Client {
    /// Favourite businesses.
    pub async fn favorites(&self) -> Result<Vec<Business>> {
        self.data("favorites", Value::Null).await
    }

    /// Adds a business to the favourites.
    pub async fn create_favorite(&self, business_public_id: &str) -> Result<()> {
        self.call_api_unit(
            "create-favorite",
            json!({ "businessPublicId": business_public_id }),
        )
        .await
    }

    /// Removes a business from the favourites.
    pub async fn delete_favorite(&self, business_public_id: &str) -> Result<()> {
        self.call_api_unit(
            "delete-favorite",
            json!({ "businessPublicId": business_public_id }),
        )
        .await
    }

    /// Saved searches that watch for new offers.
    pub async fn offer_agents(&self) -> Result<Vec<OfferAgent>> {
        self.data("offerAgents", Value::Null).await
    }

    /// Creates a saved search.
    pub async fn create_offer_agent(&self, agent: &NewOfferAgent) -> Result<CreatedPublicId> {
        self.call_api("offer-agent-create", agent).await
    }

    /// Deletes a saved search.
    pub async fn delete_offer_agent(&self, public_id: &str) -> Result<()> {
        self.call_api_unit("offer-agent-delete", json!({ "publicId": public_id }))
            .await
    }

    /// Records that a saved search was viewed.
    pub async fn touch_offer_agent(&self, public_id: &str) -> Result<()> {
        self.call_api_unit(
            "offer-agent-update",
            json!({
                "publicId": public_id,
                "lastViewedAt": crate::types::timestamp::now(),
            }),
        )
        .await
    }
}
