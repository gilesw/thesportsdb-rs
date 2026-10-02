/// Errors from either API version.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The request never produced an HTTP response.
    #[error("transport error: {0}")]
    Transport(Box<dyn std::error::Error + Send + Sync>),
    /// The server answered with a non-success status.
    #[error("TheSportsDB responded with HTTP {status}: {body}")]
    Api { status: u16, body: String },
    /// A success response whose body did not match the schema.
    #[error("TheSportsDB response did not match the schema: {0}")]
    Decode(String),
}

impl Error {
    /// True for 401 and 403, which mean the key is wrong or not entitled to
    /// the endpoint.
    pub fn is_unauthorized(&self) -> bool {
        matches!(
            self,
            Self::Api {
                status: 401 | 403,
                ..
            }
        )
    }

    /// True for 429, the quota being exhausted.
    pub fn is_rate_limited(&self) -> bool {
        matches!(self, Self::Api { status: 429, .. })
    }

    /// A success status with a body that failed to decode is a schema error.
    pub(crate) fn from_status(status: u16, body: String, parse_error: Option<String>) -> Self {
        match parse_error {
            Some(parse_error) if (200..300).contains(&status) => Self::Decode(parse_error),
            _ => Self::Api { status, body },
        }
    }
}
