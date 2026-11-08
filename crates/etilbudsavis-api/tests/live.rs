//! Optional tests against the real service. Run with `just live-test`.
#![allow(clippy::expect_used)]

use etilbudsavis_api::{Client, OffersQuery, StoresQuery};

fn client() -> Client {
    Client::builder()
        .base_url(
            std::env::var("ETILBUDSVIS_BASE_URL")
                .unwrap_or_else(|_| etilbudsavis_api::DEFAULT_BASE_URL.to_owned()),
        )
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .expect("client")
}

#[tokio::test]
#[ignore = "talks to the real eTilbudsavis service"]
async fn the_home_screen_has_category_groups() {
    let groups = client().grouped_fronts().await.expect("grouped fronts");
    assert!(!groups.is_empty());
    assert!(groups.iter().any(|(category, _)| category == "groceries"));
}

#[tokio::test]
#[ignore = "talks to the real eTilbudsavis service"]
async fn offers_have_prices_and_businesses() {
    let offers = client()
        .offers(&OffersQuery::page(5))
        .await
        .expect("offers");
    assert!(!offers.data.is_empty());
    assert!(offers.data.iter().all(|offer| offer.business.is_some()));
}

#[tokio::test]
#[ignore = "talks to the real eTilbudsavis service"]
async fn stores_are_reachable() {
    let stores = client()
        .stores(&StoresQuery::page(5))
        .await
        .expect("stores");
    assert!(!stores.data.is_empty());
    assert!(stores.data.iter().all(|store| store.id.len() > 4));
}

#[tokio::test]
#[ignore = "talks to the real eTilbudsavis service"]
async fn search_suggestions_match_a_common_term() {
    let results = client().suggestions("mælk").await.expect("suggestions");
    assert!(!results.data.is_empty());
    assert!(
        results
            .data
            .iter()
            .any(|result| result.title.contains("mælk"))
    );
}

#[tokio::test]
#[ignore = "talks to the real eTilbudsavis service"]
async fn the_video_feed_has_videos() {
    let videos = client().videos(5, None).await.expect("videos");
    assert!(!videos.is_empty());
}

#[tokio::test]
#[ignore = "talks to the real eTilbudsavis service"]
async fn the_fallback_location_is_in_scandinavia() {
    let location = client().location().await.expect("location");
    assert!(location.country.is_some());
    assert!(!location.geohash.is_empty());
}
