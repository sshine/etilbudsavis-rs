//! Catalogue: business fronts, businesses, and publications.

use serde_json::{Value, json};

use crate::client::Client;
use crate::error::Result;
use crate::types::{
    Business, BusinessesQuery, Front, FrontsQuery, GroupedFront, Publication, PublicationWrapper,
};

impl Client {
    /// All categories and the fronts they contain; the app's home screen.
    pub async fn grouped_fronts(&self) -> Result<Vec<GroupedFront>> {
        self.data("groupedFronts", Value::Null).await
    }

    /// Fronts filtered by business, category, or distance.
    pub async fn fronts(&self, query: &FrontsQuery) -> Result<Vec<Front>> {
        self.data("fronts", query).await
    }

    /// All businesses, or a filtered subset.
    pub async fn businesses(&self, query: &BusinessesQuery) -> Result<Vec<Business>> {
        self.data("businesses", query).await
    }

    /// One business by URL slug.
    pub async fn business(&self, slug: &str) -> Result<Business> {
        self.data("business", json!({ "slug": slug })).await
    }

    /// The long marketing description of a business.
    pub async fn business_description(&self, public_id: &str) -> Result<String> {
        self.data("businessDescription", json!({ "publicId": public_id }))
            .await
    }

    /// Metadata for one publication.
    pub async fn publication(&self, publication_id: &str) -> Result<PublicationWrapper> {
        self.data("publication", json!({ "publicationId": publication_id }))
            .await
    }

    /// Other publications from a business, optionally scoped to a location.
    pub async fn related_publications(
        &self,
        business_id: &str,
        location_id: Option<&str>,
    ) -> Result<Vec<Publication>> {
        let mut params = serde_json::Map::new();
        params.insert("businessId".to_owned(), json!(business_id));
        if let Some(location_id) = location_id {
            params.insert("locationId".to_owned(), json!(location_id));
        }
        self.data("relatedPublications", Value::Object(params))
            .await
    }
}
