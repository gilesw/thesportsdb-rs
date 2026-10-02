use super::generated::client::HttpClient;
use super::generated::types::{
    Event, EventLookupLookupItem, EventScheduleListScheduleItem, League,
};
use crate::error::Error;
use crate::ids::{EventId, LeagueId};

/// One row of a league schedule.
pub type ScheduleEvent = EventScheduleListScheduleItem;

/// The full record for one event.
pub type EventDetail = EventLookupLookupItem;

/// Authenticated client. Every request sends the key in the `X-API-KEY`
/// header, as the v2 API requires.
#[derive(Clone)]
pub struct Client {
    inner: HttpClient,
}

// The inner client holds the API key, so Debug shows nothing of it.
impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client").finish_non_exhaustive()
    }
}

impl Client {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            inner: HttpClient::new().with_api_key(api_key),
        }
    }

    /// Point the client at another server, such as a test double.
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.inner = self.inner.with_base_url(base_url);
        self
    }

    /// Leagues whose name contains `name`; an empty list when nothing matches.
    pub async fn search_leagues(&self, name: &str) -> Result<Vec<League>, Error> {
        let response = self.inner.search_league_by_name(name).await?;
        Ok(response.search.unwrap_or_default())
    }

    /// Events whose name contains `name`; an empty list when nothing matches.
    pub async fn search_events(&self, name: &str) -> Result<Vec<Event>, Error> {
        let response = self.inner.search_event_by_name(name).await?;
        Ok(response.search.unwrap_or_default())
    }

    /// `None` when the id is unknown.
    pub async fn league(&self, id: LeagueId) -> Result<Option<League>, Error> {
        let response = self.inner.lookup_league_by_id(id.0).await?;
        Ok(response.lookup.unwrap_or_default().into_iter().next())
    }

    /// `None` when the id is unknown.
    pub async fn event(&self, id: EventId) -> Result<Option<EventDetail>, Error> {
        let response = self.inner.lookup_event_by_id(id.0).await?;
        Ok(response.lookup.unwrap_or_default().into_iter().next())
    }

    /// Season labels for a league, in the server's order; for example
    /// `"2026"` for a calendar-year league or `"2025-2026"` for a split one.
    pub async fn seasons(&self, league: LeagueId) -> Result<Vec<String>, Error> {
        let response = self.inner.list_seasons_by_league_id(league.0).await?;
        Ok(response
            .list
            .unwrap_or_default()
            .into_iter()
            .filter_map(|item| item.str_season)
            .collect())
    }

    /// Every event of one season, past and future, in the server's order.
    pub async fn season_schedule(
        &self,
        league: LeagueId,
        season: &str,
    ) -> Result<Vec<ScheduleEvent>, Error> {
        let response = self
            .inner
            .get_full_motorsport_schedule_by_league_and_season(league.0, season)
            .await?;
        Ok(response.schedule.unwrap_or_default())
    }

    /// Upcoming events for a league. The server caps the list.
    pub async fn next_events(&self, league: LeagueId) -> Result<Vec<ScheduleEvent>, Error> {
        let response = self
            .inner
            .get_next_soccer_events_by_league(league.0)
            .await?;
        Ok(response.schedule.unwrap_or_default())
    }

    /// Recently completed events for a league. The server caps the list.
    pub async fn previous_events(&self, league: LeagueId) -> Result<Vec<ScheduleEvent>, Error> {
        let response = self
            .inner
            .get_previous_soccer_events_by_league(league.0)
            .await?;
        Ok(response.schedule.unwrap_or_default())
    }
}
