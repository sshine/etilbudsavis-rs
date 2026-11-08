//! The MCP server over eTilbudsavis.
//!
//! The binary is a thin wrapper: startup lives in [`cli`], the served surface
//! in [`server`] and [`tools`]. It is a library so integration tests can drive a
//! real client against a mock backend.

pub mod cli;
pub mod error;
pub mod render;
pub mod server;
pub mod tools;
