//! The MCP tool surface.

use etilbudsavis_api::{
    BusinessesQuery, NewOfferAgent, OffersQuery, PersonUpdate, RelatedOffersQuery,
    ShoppingListItem, ShoppingListItemUpdate, StoresQuery,
};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::error;
use crate::render;
use crate::server::EtibudsavisServer;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CatalogueParams {
    /// Restrict to one category, such as `groceries`.
    pub category: Option<String>,
    /// Maximum businesses to return.
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct BusinessesParams {
    /// Restrict to one category.
    pub category: Option<String>,
    /// Maximum rows.
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct BusinessParams {
    /// Business slug, such as `365discount`.
    pub slug: String,
    /// Also fetch the long marketing description.
    pub include_description: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct OffersParams {
    /// Business slug or public id.
    pub business: Option<String>,
    /// Free-text product search.
    pub search: Option<String>,
    /// Department slug.
    pub department: Option<String>,
    /// Sort order, such as `popularity_desc`.
    pub sort: Option<String>,
    /// Hide offers that have not started.
    pub hide_upcoming: Option<bool>,
    /// Maximum rows.
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct OfferParams {
    /// Offer public id.
    pub public_id: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct StoresParams {
    /// Business slug or public id.
    pub business: Option<String>,
    /// Publication id.
    pub publication: Option<String>,
    /// Offer public id.
    pub offer: Option<String>,
    /// Free-text store search.
    pub search: Option<String>,
    /// Maximum rows.
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct StoreParams {
    /// Store id.
    pub id: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SearchParams {
    /// The phrase.
    pub query: String,
    /// One of `suggestions`, `top`, `business`, `seo`, or `offers`.
    pub kind: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct VideosParams {
    /// Maximum rows.
    pub limit: Option<u32>,
    /// Free-text search.
    pub search: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct VideoParams {
    /// Video id.
    pub id: String,
    /// Also fetch recipe ingredients.
    pub include_ingredients: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ListItemsParams {
    /// Shopping-list id.
    pub id: i64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct PrivacyParams {
    /// Locale, such as `da` or `en`.
    pub locale: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct FavoriteParams {
    /// Business slug or public id.
    pub business: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ListCreateParams {
    /// List name.
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ListRenameParams {
    /// List id.
    pub id: i64,
    /// New name.
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ListAddParams {
    /// List id.
    pub id: i64,
    /// Item name.
    pub name: String,
    /// Quantity.
    pub count: Option<i64>,
    /// Comment or pack description.
    pub comment: Option<String>,
    /// Estimated price.
    pub price: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ListItemParams {
    /// List id.
    pub id: i64,
    /// Item client id.
    pub item: String,
    /// New name.
    pub name: Option<String>,
    /// New quantity.
    pub count: Option<i64>,
    /// Tick or untick.
    pub ticked: Option<bool>,
    /// New comment.
    pub comment: Option<String>,
    /// New estimated price.
    pub price: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ListItemRefParams {
    /// List id.
    pub id: i64,
    /// Item client id.
    pub item: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ListIdParams {
    /// List id.
    pub id: i64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct VideoIdParams {
    /// Video id.
    pub id: String,
}

#[tool_router(router = read_router, vis = "pub(crate)")]
impl EtibudsavisServer {
    /// Report whether an account token is stored and who it belongs to.
    #[tool(
        name = "etilbudsavis_session_status",
        description = "Reports whether an eTilbudsavis account token is present, who it \
                       belongs to, and whether it is anonymous. This is a cheap read; it \
                       does not create an account.",
        annotations(
            title = "Session status",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn session_status(&self) -> CallToolResult {
        if !self.client.has_token() {
            return render::json(&json!({
                "authenticated": false,
                "token_present": false,
            }));
        }
        match self.client.auth_state().await {
            Ok(state) => render::json(&json!({
                "authenticated": state.person.is_some(),
                "token_present": true,
                "person": state.person,
            })),
            Err(error) => error::failed(&error),
        }
    }

    /// The home screen: category groups and their fronts.
    #[tool(
        name = "etilbudsavis_catalogue",
        description = "Returns the catalogue home screen: category groups, their businesses, \
                       and each business's active publications with validity dates. Filter \
                       with category and cap the number of businesses with limit.",
        annotations(
            title = "Catalogue",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn catalogue(
        &self,
        Parameters(params): Parameters<CatalogueParams>,
    ) -> CallToolResult {
        let groups = match self.client.grouped_fronts().await {
            Ok(groups) => groups,
            Err(error) => return error::failed(&error),
        };
        let limit = params.limit.unwrap_or(self.limits.max_items);
        let mut items = Vec::new();
        for (category, fronts) in groups {
            if params
                .category
                .as_deref()
                .is_some_and(|want| want != category)
            {
                continue;
            }
            for front in fronts {
                for publication in &front.publications {
                    items.push(json!({
                        "category": category,
                        "business": &front.business,
                        "publication": publication,
                    }));
                }
            }
        }
        if items.len() > limit {
            items.truncate(limit);
        }
        let total = items.len();
        render::page(&items, total, self.limits)
    }

    /// All businesses, optionally filtered by category.
    #[tool(
        name = "etilbudsavis_businesses",
        description = "Lists businesses with their public ids, slugs, categories, colours, \
                       websites, and webstore flag. Optionally filter by category.",
        annotations(
            title = "Businesses",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn businesses(
        &self,
        Parameters(params): Parameters<BusinessesParams>,
    ) -> CallToolResult {
        let businesses = match self
            .client
            .businesses(&BusinessesQuery {
                category: params.category,
                limit: params.limit,
                ..BusinessesQuery::default()
            })
            .await
        {
            Ok(businesses) => businesses,
            Err(error) => return error::failed(&error),
        };
        let total = businesses.len();
        render::page(&businesses, total, self.limits)
    }

    /// One business and, optionally, its marketing description.
    #[tool(
        name = "etilbudsavis_business",
        description = "Fetches one business by slug, its logos, category, colour, website, \
                       and short description. Set include_description to fetch the long \
                       marketing description as well.",
        annotations(
            title = "Business",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn business(&self, Parameters(params): Parameters<BusinessParams>) -> CallToolResult {
        let business = match self.client.business(&params.slug).await {
            Ok(business) => business,
            Err(error) => return error::failed(&error),
        };
        if params.include_description == Some(true) {
            match self.client.business_description(&business.public_id).await {
                Ok(description) => render::json(&json!({
                    "business": business,
                    "description": description,
                })),
                Err(error) => error::failed(&error),
            }
        } else {
            render::json(&business)
        }
    }

    /// Offers, with filters.
    #[tool(
        name = "etilbudsavis_offers",
        description = "Searches current offers from leaflets and business products. Filter \
                       by business slug or public id, free text, department, sort order, or \
                       hide offers that have not started. Returns prices, unit prices, pack \
                       details, validity dates, and business summaries.",
        annotations(
            title = "Offers",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn offers(&self, Parameters(params): Parameters<OffersParams>) -> CallToolResult {
        let business_id = match &params.business {
            Some(value) => match self.business_id(value).await {
                Ok(id) => Some(vec![id]),
                Err(result) => return result,
            },
            None => None,
        };
        let query = OffersQuery {
            business_ids: business_id,
            search_term: params.search,
            department_slug: params.department,
            sort: params.sort.map(|sort| vec![sort]),
            hide_upcoming: params.hide_upcoming,
            ..OffersQuery::page(params.limit.unwrap_or(30))
        };
        match self.client.offers(&query).await {
            Ok(page) => {
                let total = page.data.len();
                render::page(&page.data, total, self.limits)
            }
            Err(error) => error::failed(&error),
        }
    }

    /// One offer.
    #[tool(
        name = "etilbudsavis_offer",
        description = "Fetches one offer by public id, including its business, validity \
                       dates, unit price, pack details, and images.",
        annotations(
            title = "Offer",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn offer(&self, Parameters(params): Parameters<OfferParams>) -> CallToolResult {
        match self.client.offer(&params.public_id).await {
            Ok(offer) => render::json(&offer),
            Err(error) => error::failed(&error),
        }
    }

    /// Offers related to an offer.
    #[tool(
        name = "etilbudsavis_related_offers",
        description = "Finds offers related to a given offer across leaflets and business \
                       products, useful for comparing the same product at other chains.",
        annotations(
            title = "Related offers",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn related_offers(
        &self,
        Parameters(params): Parameters<OfferParams>,
    ) -> CallToolResult {
        let query = RelatedOffersQuery {
            public_id: params.public_id,
            sources: Some(vec![
                "publication".to_owned(),
                "business_product".to_owned(),
            ]),
        };
        match self.client.related_offers(&query).await {
            Ok(offers) => {
                let total = offers.len();
                render::page(&offers, total, self.limits)
            }
            Err(error) => error::failed(&error),
        }
    }

    /// Stores, with filters.
    #[tool(
        name = "etilbudsavis_stores",
        description = "Lists physical stores with addresses, coordinates, opening hours, \
                       websites, and their business. Filter by business, publication, offer, \
                       or free text.",
        annotations(
            title = "Stores",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn stores(&self, Parameters(params): Parameters<StoresParams>) -> CallToolResult {
        let business_id = match &params.business {
            Some(value) => match self.business_id(value).await {
                Ok(id) => Some(id),
                Err(result) => return result,
            },
            None => None,
        };
        let query = StoresQuery {
            business_id,
            publication_id: params.publication,
            offer_id: params.offer,
            search_term: params.search,
            ..StoresQuery::page(params.limit.unwrap_or(20))
        };
        match self.client.stores(&query).await {
            Ok(page) => {
                let total = page.data.len();
                render::page(&page.data, total, self.limits)
            }
            Err(error) => error::failed(&error),
        }
    }

    /// One store.
    #[tool(
        name = "etilbudsavis_store",
        description = "Fetches one store by id, including its address, coordinates, opening \
                       hours, website, and business.",
        annotations(
            title = "Store",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn store(&self, Parameters(params): Parameters<StoreParams>) -> CallToolResult {
        match self.client.store(&params.id).await {
            Ok(store) => render::json(&store),
            Err(error) => error::failed(&error),
        }
    }

    /// Search suggestions, popular terms, a business match, SEO text, or offers.
    #[tool(
        name = "etilbudsavis_search",
        description = "Searches eTilbudsavis. kind=suggestions returns phrase suggestions; \
                       kind=top returns popular terms; kind=business returns the best \
                       business match; kind=seo returns the landing-page text and category \
                       neighbours; kind=offers (the default) searches current offers.",
        annotations(
            title = "Search",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn search(&self, Parameters(params): Parameters<SearchParams>) -> CallToolResult {
        match params.kind.as_deref().unwrap_or("offers") {
            "top" => match self.client.top_searches(20).await {
                Ok(results) => render::json(&results),
                Err(error) => error::failed(&error),
            },
            "suggestions" => match self.client.suggestions(&params.query).await {
                Ok(results) => render::json(&results),
                Err(error) => error::failed(&error),
            },
            "business" => match self.client.search_business(&params.query).await {
                Ok(result) => render::json(&result),
                Err(error) => error::failed(&error),
            },
            "seo" => match self.client.search_seo(&params.query).await {
                Ok(result) => render::json(&result),
                Err(error) => error::failed(&error),
            },
            "offers" => match self
                .client
                .offers(&OffersQuery {
                    search_term: Some(params.query),
                    ..OffersQuery::page(30)
                })
                .await
            {
                Ok(page) => {
                    let total = page.data.len();
                    render::page(&page.data, total, self.limits)
                }
                Err(error) => error::failed(&error),
            },
            other => render::rejected(format!(
                "unknown search kind `{other}`; use suggestions, top, business, seo, or offers"
            )),
        }
    }

    /// The video feed.
    #[tool(
        name = "etilbudsavis_videos",
        description = "Lists short recipe and shopping-inspiration videos with MP4 URLs, \
                       posters, authors, and country. Optionally filter by free text.",
        annotations(
            title = "Videos",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn videos(&self, Parameters(params): Parameters<VideosParams>) -> CallToolResult {
        match self
            .client
            .videos(params.limit.unwrap_or(20), params.search.as_deref())
            .await
        {
            Ok(videos) => {
                let total = videos.len();
                render::page(&videos, total, self.limits)
            }
            Err(error) => error::failed(&error),
        }
    }

    /// One video and optionally its recipe ingredients.
    #[tool(
        name = "etilbudsavis_video",
        description = "Fetches one video. Set include_ingredients to also fetch its recipe \
                       ingredients.",
        annotations(
            title = "Video",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn video(&self, Parameters(params): Parameters<VideoParams>) -> CallToolResult {
        let video = match self.client.video(&params.id).await {
            Ok(video) => video,
            Err(error) => return error::failed(&error),
        };
        if params.include_ingredients == Some(true) {
            match self.client.video_ingredients(&params.id).await {
                Ok(ingredients) => render::json(&json!({
                    "video": video,
                    "ingredients": ingredients,
                })),
                Err(error) => error::failed(&error),
            }
        } else {
            render::json(&video)
        }
    }

    /// Favourite businesses.
    #[tool(
        name = "etilbudsavis_favorites",
        description = "Lists the businesses this account marked as favourites.",
        annotations(
            title = "Favourites",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn favorites(&self) -> CallToolResult {
        match self.client.favorites().await {
            Ok(favorites) => {
                let total = favorites.len();
                render::page(&favorites, total, self.limits)
            }
            Err(error) => error::failed(&error),
        }
    }

    /// Shopping lists.
    #[tool(
        name = "etilbudsavis_shopping_lists",
        description = "Lists the shopping lists owned by or shared with the account, with \
                       their ids, names, share tokens, ownership, and collaborator counts.",
        annotations(
            title = "Shopping lists",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn shopping_lists(&self) -> CallToolResult {
        match self.client.shopping_lists().await {
            Ok(lists) => {
                let total = lists.len();
                render::page(&lists, total, self.limits)
            }
            Err(error) => error::failed(&error),
        }
    }

    /// Items on one shopping list.
    #[tool(
        name = "etilbudsavis_shopping_list_items",
        description = "Lists the items on one shopping list, with client ids, names, \
                       quantities, ticked state, comments, estimated prices, and departments.",
        annotations(
            title = "Shopping-list items",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn shopping_list_items(
        &self,
        Parameters(params): Parameters<ListItemsParams>,
    ) -> CallToolResult {
        match self.client.shopping_list_items(params.id).await {
            Ok(items) => {
                let total = items.len();
                render::page(&items, total, self.limits)
            }
            Err(error) => error::failed(&error),
        }
    }

    /// The service's fallback location.
    #[tool(
        name = "etilbudsavis_location",
        description = "Returns the location the service uses when none is supplied: \
                       coordinates, geohash, city, country, and how it was derived.",
        annotations(
            title = "Location",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn location(&self) -> CallToolResult {
        match self.client.location().await {
            Ok(location) => render::json(&location),
            Err(error) => error::failed(&error),
        }
    }

    /// The privacy policy as Markdown.
    #[tool(
        name = "etilbudsavis_privacy",
        description = "Fetches the privacy policy as Markdown for a locale, `da` by default.",
        annotations(
            title = "Privacy policy",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn privacy(&self, Parameters(params): Parameters<PrivacyParams>) -> CallToolResult {
        let locale = params.locale.unwrap_or_else(|| "da".to_owned());
        match self
            .client
            .data::<String>("privacyPolicy", json!({ "locale": locale }))
            .await
        {
            Ok(text) => render::limited(
                json!({ "locale": locale, "markdown": text }),
                self.limits.max_bytes,
            ),
            Err(error) => error::failed(&error),
        }
    }

    async fn business_id(&self, value: &str) -> Result<String, CallToolResult> {
        match self.client.business(value).await {
            Ok(business) => Ok(business.public_id),
            Err(_) => Ok(value.to_owned()),
        }
    }
}

#[tool_router(router = write_router, vis = "pub(crate)")]
impl EtibudsavisServer {
    /// Adds a favourite business.
    #[tool(
        name = "etilbudsavis_favorite_add",
        description = "Adds a business to the account's favourites. Accepts a slug or public id.",
        annotations(
            title = "Add favourite",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn favorite_add(
        &self,
        Parameters(params): Parameters<FavoriteParams>,
    ) -> CallToolResult {
        let public_id = match self.business_id(&params.business).await {
            Ok(id) => id,
            Err(result) => return result,
        };
        match self.client.create_favorite(&public_id).await {
            Ok(()) => render::json(&json!({ "added": public_id })),
            Err(error) => error::failed(&error),
        }
    }

    /// Removes a favourite business.
    #[tool(
        name = "etilbudsavis_favorite_remove",
        description = "Removes a business from the account's favourites. Accepts a slug or \
                       public id.",
        annotations(
            title = "Remove favourite",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn favorite_remove(
        &self,
        Parameters(params): Parameters<FavoriteParams>,
    ) -> CallToolResult {
        let public_id = match self.business_id(&params.business).await {
            Ok(id) => id,
            Err(result) => return result,
        };
        match self.client.delete_favorite(&public_id).await {
            Ok(()) => render::json(&json!({ "removed": public_id })),
            Err(error) => error::failed(&error),
        }
    }

    /// Creates a shopping list.
    #[tool(
        name = "etilbudsavis_shopping_list_create",
        description = "Creates an empty shopping list and returns its numeric id.",
        annotations(
            title = "Create shopping list",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    pub async fn shopping_list_create(
        &self,
        Parameters(params): Parameters<ListCreateParams>,
    ) -> CallToolResult {
        match self.client.create_shopping_list(&params.name).await {
            Ok(created) => render::json(&created),
            Err(error) => error::failed(&error),
        }
    }

    /// Renames a shopping list.
    #[tool(
        name = "etilbudsavis_shopping_list_rename",
        description = "Renames a shopping list.",
        annotations(
            title = "Rename shopping list",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn shopping_list_rename(
        &self,
        Parameters(params): Parameters<ListRenameParams>,
    ) -> CallToolResult {
        match self
            .client
            .rename_shopping_list(params.id, &params.name)
            .await
        {
            Ok(()) => render::json(&json!({ "renamed": params.id, "name": params.name })),
            Err(error) => error::failed(&error),
        }
    }

    /// Adds a free-text item to a shopping list.
    #[tool(
        name = "etilbudsavis_shopping_list_add_item",
        description = "Adds a free-text item to a shopping list. Returns the generated \
                       client id, which the update and remove tools take.",
        annotations(
            title = "Add shopping-list item",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    pub async fn shopping_list_add_item(
        &self,
        Parameters(params): Parameters<ListAddParams>,
    ) -> CallToolResult {
        let mut draft = ShoppingListItem::draft(params.name);
        if let Some(count) = params.count {
            draft = draft.count(count);
        }
        if let Some(comment) = params.comment {
            draft = draft.comment(comment);
        }
        if let Some(price) = params.price {
            draft = draft.estimated_price(price);
        }
        let client_id = draft.client_id.clone();
        match self.client.add_shopping_list_item(params.id, &draft).await {
            Ok(()) => render::json(&json!({ "id": params.id, "client_id": client_id })),
            Err(error) => error::failed(&error),
        }
    }

    /// Updates a shopping-list item.
    #[tool(
        name = "etilbudsavis_shopping_list_update_item",
        description = "Updates a shopping-list item by client id. Only the fields supplied \
                       are changed.",
        annotations(
            title = "Update shopping-list item",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn shopping_list_update_item(
        &self,
        Parameters(params): Parameters<ListItemParams>,
    ) -> CallToolResult {
        let items = match self.client.shopping_list_items(params.id).await {
            Ok(items) => items,
            Err(error) => return error::failed(&error),
        };
        let current = match items.into_iter().find(|item| item.client_id == params.item) {
            Some(item) => item,
            None => {
                return render::rejected(format!("no item {} on list {}", params.item, params.id));
            }
        };
        let mut update = ShoppingListItemUpdate::from_item(&current);
        if let Some(name) = params.name {
            update.name = name;
        }
        if let Some(count) = params.count {
            update.count = count;
        }
        if let Some(ticked) = params.ticked {
            update.ticked = ticked;
        }
        if let Some(comment) = params.comment {
            update.short_description = Some(comment);
        }
        if let Some(price) = params.price {
            update.estimated_price = Some(price);
        }
        match self
            .client
            .update_shopping_list_item(params.id, &update)
            .await
        {
            Ok(()) => render::json(&json!({ "updated": params.item })),
            Err(error) => error::failed(&error),
        }
    }

    /// Likes a video.
    #[tool(
        name = "etilbudsavis_video_like",
        description = "Adds a like to a video.",
        annotations(
            title = "Like video",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn video_like(
        &self,
        Parameters(params): Parameters<VideoIdParams>,
    ) -> CallToolResult {
        match self.client.like_video(&params.id).await {
            Ok(()) => render::json(&json!({ "liked": params.id })),
            Err(error) => error::failed(&error),
        }
    }

    /// Removes a like.
    #[tool(
        name = "etilbudsavis_video_unlike",
        description = "Removes a like from a video.",
        annotations(
            title = "Unlike video",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn video_unlike(
        &self,
        Parameters(params): Parameters<VideoIdParams>,
    ) -> CallToolResult {
        match self.client.unlike_video(&params.id).await {
            Ok(()) => render::json(&json!({ "unliked": params.id })),
            Err(error) => error::failed(&error),
        }
    }

    /// Creates a saved offer search.
    #[tool(
        name = "etilbudsavis_offer_agent_create",
        description = "Creates a saved search that watches for new matching offers.",
        annotations(
            title = "Create saved search",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    pub async fn offer_agent_create(
        &self,
        Parameters(params): Parameters<OfferAgentParams>,
    ) -> CallToolResult {
        let geohash = params.geohash.unwrap_or_else(|| "u3buy".to_owned());
        match self
            .client
            .create_offer_agent(&NewOfferAgent {
                term: params.term,
                geohash,
                sorting: Some("score_desc".to_owned()),
                max_radius: params.max_radius,
                include_upcoming: params.include_upcoming.unwrap_or(false),
                price_min: None,
                price_max: None,
                business_ids: Vec::new(),
                is_email_notifications_enabled: false,
                is_push_notifications_enabled: false,
            })
            .await
        {
            Ok(created) => render::json(&created),
            Err(error) => error::failed(&error),
        }
    }

    /// Updates the account country.
    #[tool(
        name = "etilbudsavis_account_country",
        description = "Sets the account country, which the app derives from the location.",
        annotations(
            title = "Set account country",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn account_country(
        &self,
        Parameters(params): Parameters<CountryParams>,
    ) -> CallToolResult {
        match self
            .client
            .update_person(&PersonUpdate {
                country_code: Some(params.code),
                ..PersonUpdate::default()
            })
            .await
        {
            Ok(value) => render::json(&value),
            Err(error) => error::failed(&error),
        }
    }
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct OfferAgentParams {
    /// Search term.
    pub term: String,
    /// Geohash to bind the search to.
    pub geohash: Option<String>,
    /// Maximum radius in metres.
    pub max_radius: Option<u64>,
    /// Include offers that have not started.
    pub include_upcoming: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CountryParams {
    /// ISO 3166-1 alpha-2 code.
    pub code: String,
}

#[tool_router(router = delete_router, vis = "pub(crate)")]
impl EtibudsavisServer {
    /// Deletes a shopping list.
    #[tool(
        name = "etilbudsavis_shopping_list_delete",
        description = "Deletes a shopping list for every collaborator. This cannot be undone.",
        annotations(
            title = "Delete shopping list",
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn shopping_list_delete(
        &self,
        Parameters(params): Parameters<ListIdParams>,
    ) -> CallToolResult {
        match self.client.delete_shopping_list(params.id).await {
            Ok(()) => render::json(&json!({ "deleted": params.id })),
            Err(error) => error::failed(&error),
        }
    }

    /// Removes one item from a shopping list.
    #[tool(
        name = "etilbudsavis_shopping_list_remove_item",
        description = "Removes one item from a shopping list by client id.",
        annotations(
            title = "Remove shopping-list item",
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn shopping_list_remove_item(
        &self,
        Parameters(params): Parameters<ListItemRefParams>,
    ) -> CallToolResult {
        match self
            .client
            .remove_shopping_list_item(params.id, &params.item)
            .await
        {
            Ok(()) => render::json(&json!({ "removed": params.item })),
            Err(error) => error::failed(&error),
        }
    }

    /// Removes every item from a shopping list.
    #[tool(
        name = "etilbudsavis_shopping_list_clear",
        description = "Removes every item from a shopping list.",
        annotations(
            title = "Clear shopping list",
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn shopping_list_clear(
        &self,
        Parameters(params): Parameters<ListIdParams>,
    ) -> CallToolResult {
        let items = match self.client.shopping_list_items(params.id).await {
            Ok(items) => items,
            Err(error) => return error::failed(&error),
        };
        let ids: Vec<String> = items.into_iter().map(|item| item.client_id).collect();
        match self
            .client
            .remove_shopping_list_items(params.id, &ids)
            .await
        {
            Ok(()) => render::json(&json!({ "removed": ids.len() })),
            Err(error) => error::failed(&error),
        }
    }
}
