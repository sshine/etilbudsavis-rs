//! A typed async client for the backend behind the eTilbudsavis Android app.
//!
//! The app is a Capacitor shell around a React web application. Every read goes to
//! `POST /` as a batch of base64-encoded `[dataKey, params]` tuples and comes back
//! as newline-delimited JSON; every command goes to `POST /api/<operation>`. Both
//! carry the public Android `x-api-key`, and account commands add `x-token`.
//!
//! ```
//! # async fn example() -> Result<(), etilbudsavis_api::Error> {
//! let client = etilbudsavis_api::Client::anonymous()?;
//! let fronts = client.grouped_fronts().await?;
//! println!("{} categories", fronts.len());
//! # Ok(())
//! # }
//! ```

pub mod auth;
pub mod catalogue;
pub mod client;
pub mod error;
pub mod favorites;
pub mod location;
pub mod offers;
pub mod search;
pub mod session;
pub mod shopping_lists;
pub mod stores;
pub mod types;
pub mod videos;

pub use client::{Client, ClientBuilder, DEFAULT_API_KEY, DEFAULT_BASE_URL, DEFAULT_USER_AGENT};
pub use error::{Error, Result};
pub use session::{Session, StoredSession};
pub use types::*;
