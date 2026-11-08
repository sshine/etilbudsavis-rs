//! Command line, environment, and client construction.

use std::io::{self, IsTerminal as _};
use std::path::PathBuf;
use std::time::Duration;

use chrono::Utc;
use clap::Parser;
use directories::ProjectDirs;
use etilbudsavis_api::{Client, Session, StoredSession};
use tracing_subscriber::EnvFilter;

use crate::render::OutputLimits;
use crate::server::Mode;

/// Serve eTilbudsavis to an MCP client over stdio.
#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Args {
    /// Where the account token is stored.
    #[arg(long, env = "ETILBUDSVIS_SESSION_FILE")]
    session_file: Option<PathBuf>,

    /// Account token. Defaults to the token file, then ETILBUDSVIS_TOKEN.
    #[arg(long, env = "ETILBUDSVIS_TOKEN")]
    token: Option<String>,

    /// Base URL, for a staging host or a test server.
    #[arg(long, env = "ETILBUDSVIS_BASE_URL", hide = true)]
    base_url: Option<String>,

    /// Android API key override.
    #[arg(long, env = "ETILBUDSVIS_API_KEY", hide = true)]
    api_key: Option<String>,

    /// Seconds to wait for one request.
    #[arg(long, default_value_t = 30)]
    timeout: u64,

    /// Rows a listing returns before it truncates.
    #[arg(long, default_value_t = 100)]
    max_items: usize,

    /// Bytes a tool result may reach before it truncates.
    #[arg(long, default_value_t = 100_000)]
    max_bytes: usize,

    /// Also serve the tools that change favourites, lists, or likes.
    #[arg(long)]
    pub allow_writes: bool,

    /// Also serve the tools that delete lists or remove items.
    ///
    /// This needs --allow-writes as well.
    #[arg(long)]
    pub allow_deletes: bool,

    /// Print the tool schemas as JSON and exit.
    #[arg(long)]
    pub list_tools: bool,

    /// Sign in and exit without serving.
    #[arg(long)]
    pub check: bool,
}

impl Args {
    /// Which groups of tools to serve.
    pub fn mode(&self) -> Result<Mode, StartupError> {
        if self.allow_deletes && !self.allow_writes {
            return Err(StartupError::DeletesWithoutWrites);
        }
        Ok(Mode {
            writes: self.allow_writes,
            deletes: self.allow_deletes,
        })
    }

    /// How much one tool may return.
    pub fn limits(&self) -> OutputLimits {
        OutputLimits {
            max_items: self.max_items,
            max_bytes: self.max_bytes,
        }
    }

    /// Build the one client the whole process shares.
    ///
    /// When writes are enabled and no token exists, this creates an anonymous
    /// account the way the app does, so list tools work without a sign-up.
    pub async fn client(&self) -> Result<Client, StartupError> {
        let mut builder = Client::builder()
            .timeout(Duration::from_secs(self.timeout))
            .api_key(
                self.api_key
                    .clone()
                    .unwrap_or_else(|| etilbudsavis_api::DEFAULT_API_KEY.to_owned()),
            );
        if let Some(base_url) = &self.base_url {
            builder = builder.base_url(base_url);
        }

        let session = Session::at(
            self.session_file
                .clone()
                .unwrap_or_else(default_session_file),
        );
        let token = self.token.clone().or_else(|| {
            session
                .load()
                .ok()
                .flatten()
                .map(|stored| stored.token.expose().to_owned())
        });

        if let Some(token) = token {
            builder = builder.token(token);
            return builder.build().map_err(StartupError::Client);
        }

        let client = builder.clone().build().map_err(StartupError::Client)?;
        if self.allow_writes {
            let anonymous = client
                .create_anonymous()
                .await
                .map_err(|error| StartupError::Rejected(Box::new(error)))?;
            let token = anonymous.token.clone();
            session
                .save(&StoredSession {
                    token: anonymous.token,
                    email: anonymous.email,
                    created_at: Utc::now(),
                })
                .map_err(StartupError::Session)?;
            return builder.token(token).build().map_err(StartupError::Client);
        }

        Ok(client)
    }
}

fn default_session_file() -> PathBuf {
    ProjectDirs::from("dk", "sshine", "etilbudsavis")
        .map(|dirs| dirs.data_dir().join("token.json"))
        .unwrap_or_else(|| PathBuf::from("etilbudsavis-token.json"))
}

/// What went wrong before the first request.
#[derive(Debug, thiserror::Error)]
pub enum StartupError {
    /// The client could not be built.
    #[error("could not build the API client")]
    Client(#[source] etilbudsavis_api::Error),

    /// The account could not be created or read.
    #[error("eTilbudsavis did not accept the account request")]
    Rejected(#[source] Box<etilbudsavis_api::Error>),

    /// The token file could not be read or written.
    #[error("could not use the token file")]
    Session(#[source] etilbudsavis_api::Error),

    /// Deleting was requested without writes.
    #[error("--allow-deletes needs --allow-writes as well; pass both or neither")]
    DeletesWithoutWrites,

    /// The tool schemas could not be serialized.
    #[error("could not write the tool schemas")]
    Schemas(#[from] serde_json::Error),
}

/// Send the log to stderr, because stdout carries the protocol.
pub fn init_tracing() {
    let filter = EnvFilter::try_from_env("ETILBUDSVIS_MCP_LOG")
        .or_else(|_| EnvFilter::try_from_default_env())
        .unwrap_or_else(|_| EnvFilter::new("etilbudsavis_mcp=info,etilbudsavis_api=warn"));

    tracing_subscriber::fmt()
        .with_writer(io::stderr)
        .with_ansi(io::stderr().is_terminal())
        .with_env_filter(filter)
        .init();
}
