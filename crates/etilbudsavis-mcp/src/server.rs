//! The MCP server itself.

use rmcp::ServerHandler;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::model::{Implementation, ServerCapabilities, ServerConfig};
use rmcp::tool_handler;

use crate::render::OutputLimits;

/// Which groups of tools to serve.
///
/// Reads are always served. Writes add favourites, shopping-list edits, and
/// video likes. Deletes add list deletion and item removal and require writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Mode {
    /// Serve the tools that change favourites, lists, or likes.
    pub writes: bool,
    /// Serve the tools that delete lists or remove items.
    pub deletes: bool,
}

/// One eTilbudsavis session, served over MCP.
#[derive(Debug, Clone)]
pub struct EtibudsavisServer {
    pub(crate) client: etilbudsavis_api::Client,
    pub(crate) limits: OutputLimits,
    mode: Mode,
    tool_router: ToolRouter<Self>,
}

impl EtibudsavisServer {
    /// Builds a server around one client.
    pub fn new(client: etilbudsavis_api::Client, limits: OutputLimits, mode: Mode) -> Self {
        Self {
            client,
            limits,
            mode,
            tool_router: Self::router(mode),
        }
    }

    /// The tools this mode serves.
    pub fn router(mode: Mode) -> ToolRouter<Self> {
        let mut router = Self::read_router();
        if mode.writes {
            router.merge(Self::write_router());
        }
        if mode.deletes {
            router.merge(Self::delete_router());
        }
        router
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for EtibudsavisServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new(
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION"),
            ))
            .with_instructions(instructions(self.mode))
    }
}

fn instructions(mode: Mode) -> String {
    let mut text = String::from(
        "This server exposes eTilbudsavis, the Danish leaflet and offer aggregator \
         behind the eTilbudsavis Android app. Read tools cover catalogue fronts, \
         businesses, publications, offers, stores, search, videos, favourites, shopping \
         lists, and the service's fallback location. Public catalogue reads need no \
         account; favourites and lists use the token in the XDG data directory, and the \
         server creates an anonymous account when it starts with --allow-writes and no \
         token exists.",
    );

    if !mode.writes {
        text.push_str(
            " This server was started without --allow-writes, so it serves no tool that \
             changes an account.",
        );
    } else if !mode.deletes {
        text.push_str(
            " Write tools are enabled for favourites, shopping-list edits, and video likes. \
             Deleting a list or removing an item is a separate tier behind --allow-deletes.",
        );
    } else {
        text.push_str(
            " Write and delete tools are enabled. Deleting a shopping list is destructive \
             and removes it for every collaborator.",
        );
    }

    text
}
