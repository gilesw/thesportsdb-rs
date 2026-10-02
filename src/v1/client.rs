use super::generated::client::HttpClient;
use super::generated::types::{EventSchema, LeaguesSchema};
use crate::error::Error;
use crate::ids::LeagueId;

/// One event row, as returned by the schedule endpoints.
pub type Event = EventSchema;

/// One league record.
pub type League = LeaguesSchema;

/// TheSportsDB's public test key. It works with the v1 API, which then caps
/// every list, for example to one upcoming and one past event.
pub const FREE_KEY: &str = "123";

const BASE: &str = "https://www.thesportsdb.com/api/v1/json";

/// The v1 API carries the key as a path segment, so it is part of the base
/// URL rather than a header.
#[derive(Clone)]
pub struct Client {
    inner: HttpClient,
}

// The base URL holds the API key, so Debug shows nothing of it.
impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client").finish_non_exhaustive()
    }
}

impl Client {
    /// A key that is empty or not a single path segment is replaced by
    /// [`FREE_KEY`].
    pub fn new(api_key: &str) -> Self {
        let key = api_key.trim();
        let key = if key.is_empty() || !key.chars().all(|c| c.is_ascii_alphanumeric()) {
            FREE_KEY
        } else {
            key
        };
        Self::with_base_url(format!("{BASE}/{key}"))
    }

    /// `base_url` already includes the key segment, for example
    /// `https://www.thesportsdb.com/api/v1/json/123` or a test double.
    pub fn with_base_url(base_url: impl Into<String>) -> Self {
        Self {
            inner: HttpClient::new().with_base_url(base_url),
        }
    }

    /// `None` when the id is unknown.
    pub async fn league(&self, id: LeagueId) -> Result<Option<League>, Error> {
        let response = self.inner.get_league_by_id(id.0).await?;
        Ok(response.leagues.unwrap_or_default().into_iter().next())
    }

    /// Season labels for a league in the server's order, such as `"2026"`.
    pub async fn seasons(&self, league: LeagueId) -> Result<Vec<String>, Error> {
        let response = self
            .inner
            .search_all_seasons(league.0.to_string(), None, None)
            .await?;
        Ok(response
            .seasons
            .unwrap_or_default()
            .into_iter()
            .filter_map(|item| item.str_season)
            .collect())
    }

    /// Events of one season in the server's order.
    pub async fn season_events(&self, league: LeagueId, season: &str) -> Result<Vec<Event>, Error> {
        let response = self.inner.get_events_by_season(league.0, season).await?;
        Ok(response.events.unwrap_or_default())
    }

    /// The league's upcoming events.
    pub async fn next_events(&self, league: LeagueId) -> Result<Vec<Event>, Error> {
        let response = self.inner.get_next_league_events(league.0).await?;
        Ok(response.events.unwrap_or_default())
    }

    /// The league's most recent events.
    pub async fn previous_events(&self, league: LeagueId) -> Result<Vec<Event>, Error> {
        let response = self.inner.get_past_league_events(league.0).await?;
        Ok(response.events.unwrap_or_default())
    }
}
