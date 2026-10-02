//! TheSportsDB v2.
//!
//! The key is sent in the `X-API-KEY` header and must be a paid key.

mod client;

// Kept byte-identical to the generator output so its --check mode works. The
// generator also writes a mod.rs there, which is not used.
#[rustfmt::skip]
#[allow(dead_code, clippy::all, clippy::pedantic, missing_docs, rustdoc::all)]
pub mod generated {
    pub mod client;
    pub mod types;
}

pub use client::{Client, EventDetail, ScheduleEvent};
pub use generated::types::{Event, League};

impl<E: std::fmt::Debug> From<generated::client::ApiOpError<E>> for crate::Error {
    fn from(error: generated::client::ApiOpError<E>) -> Self {
        match error {
            generated::client::ApiOpError::Transport(e) => Self::Transport(Box::new(e)),
            generated::client::ApiOpError::Api(e) => {
                Self::from_status(e.status, e.body, e.parse_error)
            }
        }
    }
}
