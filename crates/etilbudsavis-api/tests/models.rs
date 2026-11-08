//! Deserialization of representative payloads captured from the service.
#![allow(clippy::expect_used)]

use etilbudsavis_api::{
    Business, GroupedFront, Offer, Paginated, Person, ShoppingList, ShoppingListItem, Store,
    TopSearch, Video,
};

#[test]
fn a_business_ignores_fields_the_client_does_not_need() {
    let business: Business = serde_json::from_str(
        r##"{
            "publicId": "DWZE1w",
            "name": "365discount",
            "slugs": ["365discount"],
            "countryCode": "DK",
            "positiveLogoImage": "https://example.test/positive.png",
            "negativeLogoImage": "https://example.test/negative.png",
            "positiveLogoMark": "https://example.test/mark.png",
            "negativeLogoMark": "https://example.test/mark-dark.png",
            "primaryCategoryId": "groceries",
            "primaryColor": "#01AA46",
            "isStatic": false,
            "isEnabled": true,
            "shortDescription": "Danmarks nyeste discountkæde",
            "website": "https://example.test/",
            "isWebstore": false,
            "somethingTheServerAddedYesterday": 42
        }"##,
    )
    .expect("business");
    assert_eq!(business.public_id, "DWZE1w");
    assert_eq!(business.slug(), Some("365discount"));
    assert_eq!(business.primary_category_id.as_deref(), Some("groceries"));
}

#[test]
fn an_offer_carries_prices_pack_details_and_dates() {
    let offer: Offer = serde_json::from_str(
        r##"{
            "publicId": "4RGZi8r9lgGGiGo6GrN87",
            "name": "Lurpak smør eller smørbar",
            "description": "200 g. Kg-pris 60,00.",
            "currencyCode": "DKK",
            "unitPrice": 60,
            "baseUnit": "kilogram",
            "price": 12,
            "savings": null,
            "fromPrice": null,
            "membershipPrice": null,
            "appPrice": null,
            "relativeSavings": null,
            "membershipRelativeSavings": null,
            "pieceCountGet": null,
            "pieceCountGetFor": null,
            "webshopLink": null,
            "image": "https://example.test/small.png",
            "imageLarge": "https://example.test/large.png",
            "validFrom": "2026-11-04T23:00:00+0000",
            "validUntil": "2026-11-11T22:59:59+0000",
            "visibleFrom": "2026-11-02T23:01:00+0000",
            "displayValidUntil": true,
            "business": {
                "publicId": "DWZE1w",
                "name": "365discount",
                "slugs": ["365discount"],
                "countryCode": "DK",
                "isStatic": false,
                "isEnabled": true,
                "primaryCategoryId": "groceries",
                "primaryColor": "#01AA46",
                "website": "https://example.test/",
                "isWebstore": false
            },
            "businessPublicId": "DWZE1w",
            "publicationPublicId": "NZ6IiwsU",
            "publicationCoverImage": "https://example.test/cover.png",
            "unitSymbol": "g",
            "pieceCountFrom": 1,
            "pieceCountTo": 1,
            "pieceCountMin": null,
            "pieceCountMax": 3,
            "unitSizeFrom": 200,
            "unitSizeTo": 200,
            "departmentSlug": "dairy-and-cold"
        }"##,
    )
    .expect("offer");
    assert_eq!(offer.price, Some(12.0));
    assert_eq!(offer.unit_price, Some(60.0));
    assert_eq!(offer.piece_count_max, Some(3));
    assert_eq!(offer.department_slug.as_deref(), Some("dairy-and-cold"));
    assert!(offer.valid_from.is_some());
    // The service writes `+0000`, which is not strict RFC 3339.
    assert_eq!(
        offer
            .valid_until
            .expect("valid until")
            .format("%Y-%m-%d")
            .to_string(),
        "2026-11-11"
    );
}

#[test]
fn a_store_decodes_opening_hours_and_coordinates() {
    let store: Store = serde_json::from_str(
        r##"{
            "id": "MPEZT66aHali9_P7mKHrr",
            "name": "MyTrendyPhone",
            "openingHours": [
                { "opens": "07:00:00", "closes": "22:00:00", "day_of_week": "sunday" }
            ],
            "postalCode": "1171",
            "countryCode": "DK",
            "city": "København",
            "streetAddress": "Fiolstræde 17C",
            "coordinates": { "latitude": 55.681721, "longitude": 12.572136 },
            "website": "https://example.test/",
            "business": {
                "publicId": "9ohkID",
                "name": "MyTrendyPhone",
                "slugs": ["MyTrendyPhone"],
                "countryCode": "DK",
                "isStatic": false,
                "isEnabled": true,
                "primaryCategoryId": "others",
                "primaryColor": "#000000",
                "isWebstore": false
            }
        }"##,
    )
    .expect("store");
    assert_eq!(
        store.opening_hours[0].day_of_week.as_deref(),
        Some("sunday")
    );
    assert_eq!(
        store.coordinates.map(|coordinates| coordinates.latitude),
        Some(55.681721)
    );
}

#[test]
fn a_shopping_list_and_its_items_decode_the_service_snake_case() {
    let lists: Vec<ShoppingList> = serde_json::from_str(
        r##"[{
            "id": 6567916,
            "name": "my test list",
            "created_at": "2026-11-08T11:47:33.096Z",
            "updated_at": "2026-11-08T11:47:33.096Z",
            "share_token": "SHARETOKEN",
            "owned_by_you": true,
            "collaborator_count": 0
        }]"##,
    )
    .expect("lists");
    assert!(lists[0].owned_by_you);
    assert_eq!(lists[0].collaborator_count, 0);

    let items: Vec<ShoppingListItem> = serde_json::from_str(
        r##"[{
            "id": 235927478,
            "businessPublicId": null,
            "clientId": "89157002-926e-4076-9d19-924c8b338b6b",
            "previousClientId": null,
            "createdAt": "2026-11-08T11:47:38.711Z",
            "updatedAt": "2026-11-08T11:47:38.711Z",
            "name": "mælk",
            "shortDescription": "probe",
            "ticked": false,
            "count": 2,
            "estimatedPrice": null,
            "isActive": true,
            "departmentSlug": null
        }]"##,
    )
    .expect("items");
    assert_eq!(items[0].name, "mælk");
    assert_eq!(items[0].count, 2);
    assert_eq!(items[0].short_description.as_deref(), Some("probe"));
}

#[test]
fn a_person_and_a_paginated_list_decode() {
    let person: Person = serde_json::from_str(
        r##"{
            "id": "1000001",
            "is_anonymous": true,
            "name": null,
            "email": null,
            "is_email_verified": false,
            "avatar_image": null,
            "bio": "",
            "flags": [],
            "country_code": "DK",
            "privacy_policy_consented": null,
            "enabled_notifications": []
        }"##,
    )
    .expect("person");
    assert!(person.is_anonymous);

    let searches: Paginated<TopSearch> = serde_json::from_str(
        r##"{
            "data": [{ "title": "mælk", "type": "search", "source": "search" }],
            "_metadata": { "pagination": { "limit": 3, "offset": 0 } }
        }"##,
    )
    .expect("searches");
    assert_eq!(searches.data[0].title, "mælk");
    assert_eq!(
        searches
            .metadata
            .and_then(|metadata| metadata.pagination)
            .map(|pagination| pagination.offset),
        Some(0)
    );
}

#[test]
fn grouped_fronts_are_category_and_front_pairs() {
    let groups: Vec<GroupedFront> = serde_json::from_str(
        r##"[[
            "groceries",
            [{
                "category": "groceries",
                "business": {
                    "publicId": "i7NdvM",
                    "name": "Wolt Market",
                    "slugs": ["Wolt-Market"],
                    "countryCode": "DK",
                    "isStatic": false,
                    "isEnabled": true,
                    "primaryCategoryId": "groceries",
                    "primaryColor": "#000000",
                    "isWebstore": false
                },
                "publications": [{
                    "id": "qteHWajM",
                    "label": "Tilbudsavis uge 45",
                    "thumb": "https://example.test/thumb.png",
                    "view": "https://example.test/view.png",
                    "validFrom": "2026-11-02T23:00:00.000Z",
                    "validUntil": "2026-11-09T23:00:00.000Z",
                    "displayValidUntil": true,
                    "publish": "2026-11-01T08:00:43.200Z"
                }]
            }]
        ]]"##,
    )
    .expect("grouped fronts");
    assert_eq!(groups[0].0, "groceries");
    assert_eq!(groups[0].1[0].business.name, "Wolt Market");
    assert_eq!(groups[0].1[0].publications[0].id, "qteHWajM");
}

#[test]
fn a_video_keeps_its_author_and_recipe_as_sent() {
    let video: Video = serde_json::from_str(
        r##"{
            "id": "w270YW3_",
            "createdAt": "2026-11-06T12:15:32.674Z",
            "type": "unknown",
            "name": "Tilbuds haul fra 365",
            "description": "Har jagtet gode tilbud",
            "video1080p": "https://example.test/1080.mp4",
            "video720p": "https://example.test/720.mp4",
            "poster": "https://example.test/poster.jpg",
            "posterThumb": "https://example.test/thumb.jpg",
            "author": {
                "name": "Nicole Ryle",
                "image": "https://example.test/avatar.jpg",
                "bio": "Mor til 3"
            },
            "countryCode": "DK",
            "recipe": { "yield": 1 },
            "likedByUser": false
        }"##,
    )
    .expect("video");
    assert_eq!(video.name, "Tilbuds haul fra 365");
    assert_eq!(
        video.author.and_then(|author| author.name).as_deref(),
        Some("Nicole Ryle")
    );
    assert!(video.created_at.is_some());
}
