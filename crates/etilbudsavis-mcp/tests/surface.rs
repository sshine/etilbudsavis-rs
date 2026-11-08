//! The rules the MCP tool surface has to keep.
#![allow(clippy::expect_used)]

use etilbudsavis_mcp::server::{EtibudsavisServer, Mode};

#[test]
fn every_tool_states_its_annotations() {
    for tool in EtibudsavisServer::router(Mode::default()).list_all() {
        let name = &tool.name;
        let annotations = tool
            .annotations
            .unwrap_or_else(|| panic!("{name} has no annotations"));
        assert!(annotations.title.is_some(), "{name} has no title");
        assert_eq!(
            annotations.read_only_hint,
            Some(true),
            "{name} is not read-only"
        );
        assert_eq!(
            annotations.destructive_hint,
            Some(false),
            "{name} claims to be destructive"
        );
        assert_eq!(
            annotations.open_world_hint,
            Some(true),
            "{name} does not say it reaches the service"
        );
    }
}

#[test]
fn names_are_prefixed_and_lowercase() {
    for tool in EtibudsavisServer::router(Mode {
        writes: true,
        deletes: true,
    })
    .list_all()
    {
        let name = tool.name.as_ref();
        assert!(name.starts_with("etilbudsavis_"), "{name} is not prefixed");
        assert!(name.len() <= 64, "{name} is too long");
        assert!(
            name.bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'),
            "{name} is not lowercase_with_underscores"
        );
    }
}

#[test]
fn every_tool_and_argument_is_described() {
    for tool in EtibudsavisServer::router(Mode {
        writes: true,
        deletes: true,
    })
    .list_all()
    {
        let name = &tool.name;
        assert!(
            tool.description
                .as_deref()
                .is_some_and(|description| !description.trim().is_empty()),
            "{name} has no description"
        );

        let Some(properties) = tool
            .input_schema
            .get("properties")
            .and_then(|properties| properties.as_object())
        else {
            continue;
        };
        for (argument, schema) in properties {
            assert!(
                schema
                    .get("description")
                    .and_then(|description| description.as_str())
                    .is_some_and(|description| !description.trim().is_empty()),
                "{name}.{argument} has no description"
            );
        }
    }
}

#[test]
fn the_read_only_surface_has_the_catalogue_tools() {
    let names: Vec<String> = EtibudsavisServer::router(Mode::default())
        .list_all()
        .into_iter()
        .map(|tool| tool.name.into())
        .collect();
    for expected in [
        "etilbudsavis_session_status",
        "etilbudsavis_catalogue",
        "etilbudsavis_businesses",
        "etilbudsavis_business",
        "etilbudsavis_offers",
        "etilbudsavis_offer",
        "etilbudsavis_stores",
        "etilbudsavis_search",
        "etilbudsavis_videos",
        "etilbudsavis_favorites",
        "etilbudsavis_shopping_lists",
        "etilbudsavis_location",
    ] {
        assert!(names.contains(&expected.to_owned()), "missing {expected}");
    }
}

#[test]
fn writes_and_deletes_are_separate_tiers() {
    let reads = EtibudsavisServer::router(Mode::default()).list_all().len();
    let writes = EtibudsavisServer::router(Mode {
        writes: true,
        deletes: false,
    })
    .list_all()
    .len();
    let deletes = EtibudsavisServer::router(Mode {
        writes: true,
        deletes: true,
    })
    .list_all()
    .len();
    assert!(writes > reads);
    assert!(deletes > writes);

    let write_tools = EtibudsavisServer::router(Mode {
        writes: true,
        deletes: false,
    })
    .list_all();
    assert!(
        !write_tools
            .iter()
            .any(|tool| tool.name == "etilbudsavis_shopping_list_delete")
    );

    let delete_tools = EtibudsavisServer::router(Mode {
        writes: true,
        deletes: true,
    })
    .list_all();
    let delete = delete_tools
        .into_iter()
        .find(|tool| tool.name == "etilbudsavis_shopping_list_delete")
        .expect("the delete tool is served");
    let annotations = delete.annotations.expect("annotations");
    assert_eq!(annotations.destructive_hint, Some(true));
    assert_eq!(annotations.read_only_hint, Some(false));
}
