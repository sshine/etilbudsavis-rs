//! Physical stores.

use serde_json::json;

use crate::client::Client;
use crate::error::Result;
use crate::types::{Paginated, Store, StoresQuery};

impl Client {
    /// One store by id.
    pub async fn store(&self, store_id: &str) -> Result<Store> {
        self.data("store", json!({ "storeId": store_id })).await
    }

    /// Stores filtered by business, publication, offer, or free text.
    pub async fn stores(&self, query: &StoresQuery) -> Result<Paginated<Store>> {
        self.data("stores", query).await
    }

    /// Every store matching a query, up to `max` rows.
    pub async fn all_stores(&self, mut query: StoresQuery, max: u32) -> Result<Vec<Store>> {
        let mut stores = Vec::new();
        let mut pagination = query.pagination.unwrap_or_default();
        pagination.offset = 0;
        loop {
            pagination.limit = pagination
                .limit
                .min(max.saturating_sub(stores.len() as u32));
            if pagination.limit == 0 {
                break;
            }
            query.pagination = Some(pagination);
            let page = self.stores(&query).await?;
            let received = page.data.len();
            stores.extend(page.data);
            if received == 0 || stores.len() as u32 >= max {
                break;
            }
            pagination = pagination.next();
        }
        Ok(stores)
    }
}
