//! Clients for the TheSportsDB JSON API, one module per API version.
//!
//! Each version's `generated` module is produced by openapi-to-rust from
//! `specs/thesportsdb-<version>.yaml` and is not edited by hand. Each `Client`
//! wraps it with the key, typed ids and methods that unwrap the response
//! envelopes. Ids and errors are shared between versions.

#![forbid(unsafe_code)]

mod error;
mod ids;
pub mod v1;
pub mod v2;

pub use error::Error;
pub use ids::{EventId, LeagueId};
