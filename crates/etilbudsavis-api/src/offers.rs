//! Offers, offer products, and department groupings.

use serde_json::{Value, json};

use crate::client::Client;
use crate::error::Result;
use crate::types::{
    DepartmentOffers, ForYou, Offer, OfferProduct, OffersQuery, Paginated, RelatedOffersQuery,
};

impl Client {
    /// A page of offers.
    pub async fn offers(&self, query: &OffersQuery) -> Result<Paginated<Offer>> {
        self.data("offers", query).await
    }

    /// Every offer across every page, up to `max` rows.
    pub async fn all_offers(&self, mut query: OffersQuery, max: u32) -> Result<Vec<Offer>> {
        let mut offers = Vec::new();
        let mut pagination = query.pagination.unwrap_or_default();
        pagination.offset = 0;
        loop {
            pagination.limit = pagination
                .limit
                .min(max.saturating_sub(offers.len() as u32));
            if pagination.limit == 0 {
                break;
            }
            query.pagination = Some(pagination);
            let page = self.offers(&query).await?;
            let received = page.data.len();
            offers.extend(page.data);
            if received == 0 || offers.len() as u32 >= max {
                break;
            }
            pagination = pagination.next();
        }
        Ok(offers)
    }

    /// One offer by public id.
    pub async fn offer(&self, public_id: &str) -> Result<Offer> {
        self.data("offer", json!({ "publicId": public_id })).await
    }

    /// Product variants behind an offer.
    pub async fn offer_products(&self, public_id: &str) -> Result<Vec<OfferProduct>> {
        self.data("offerProducts", json!({ "publicId": public_id }))
            .await
    }

    /// Offers related to another offer.
    pub async fn related_offers(&self, query: &RelatedOffersQuery) -> Result<Vec<Offer>> {
        self.data("relatedOffers", query).await
    }

    /// Offers grouped by department, as the in-store browsing view shows them.
    pub async fn department_offers(
        &self,
        query: &OffersQuery,
    ) -> Result<Paginated<DepartmentOffers>> {
        self.data("departmentOffers", query).await
    }

    /// The personalised "for you" row.
    pub async fn for_you(&self) -> Result<ForYou> {
        self.data("forYou", Value::Null).await
    }
}
