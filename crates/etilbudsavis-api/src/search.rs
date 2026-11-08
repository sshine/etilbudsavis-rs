//! Search suggestions and SEO landing pages.

use serde::Serialize;
use serde_json::{Value, json};

use crate::client::Client;
use crate::error::Result;
use crate::types::{Business, Paginated, SearchBusiness, SearchResults, SearchSeo, TopSearch};

/// The suggestion kinds the `search` command accepts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchType {
    /// Popular and matching search phrases.
    SearchQuery,
    /// Names used for shopping-list items.
    ShoppingListItem,
}

/// A suggestion request.
#[derive(Debug, Clone, Serialize)]
pub struct Suggestions {
    /// The text typed so far.
    pub query: String,
    /// Which suggestion index to consult.
    pub types: Vec<SearchType>,
}

impl Client {
    /// Search suggestions for a phrase.
    pub async fn suggestions(&self, query: &str) -> Result<SearchResults> {
        let request = Suggestions {
            query: query.to_owned(),
            types: vec![SearchType::SearchQuery, SearchType::ShoppingListItem],
        };
        self.call_api("search", request).await
    }

    /// Shopping-list item suggestions for a phrase.
    pub async fn shopping_list_suggestions(&self, query: &str) -> Result<SearchResults> {
        let request = Suggestions {
            query: query.to_owned(),
            types: vec![SearchType::ShoppingListItem],
        };
        self.call_api("search", request).await
    }

    /// Popular search terms.
    pub async fn top_searches(&self, limit: u32) -> Result<Paginated<TopSearch>> {
        self.data(
            "topSearches",
            json!({ "pagination": { "limit": limit, "offset": 0 } }),
        )
        .await
    }

    /// The business whose name best matches a search term.
    pub async fn search_business(&self, term: &str) -> Result<SearchBusiness> {
        self.data("searchBusiness", json!({ "searchTerm": term }))
            .await
    }

    /// SEO landing-page content for a search term.
    pub async fn search_seo(&self, term: &str) -> Result<SearchSeo> {
        self.data("searchSEO", json!({ "searchTerm": term })).await
    }

    /// The `searchBusiness` command, but returning only the business.
    pub async fn find_business(&self, term: &str) -> Result<Option<Business>> {
        Ok(self.search_business(term).await?.business)
    }

    /// Raw `search` command response, for callers exploring new result kinds.
    pub async fn search_raw(&self, body: Value) -> Result<Value> {
        self.call_api_value("search", body).await
    }
}
