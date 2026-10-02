//! The fixtures are the response examples from the published v2 OpenAPI
//! document, not captures from the live API.

use thesportsdb::v2::Client;
use thesportsdb::{Error, EventId, LeagueId};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const KEY: &str = "test-key";

fn fixture(name: &str) -> String {
    let file = format!(
        "{}/tests/fixtures/v2/{name}.json",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&file).unwrap_or_else(|e| panic!("{file}: {e}"))
}

async fn server_with(route: &str, name: &str) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(route))
        .and(header("X-API-KEY", KEY))
        .respond_with(ResponseTemplate::new(200).set_body_string(fixture(name)))
        .expect(1)
        .mount(&server)
        .await;
    server
}

fn client(server: &MockServer) -> Client {
    Client::new(KEY).with_base_url(server.uri())
}

#[tokio::test]
async fn search_leagues_sends_key_and_unwraps_envelope() {
    let server = server_with("/search/league/Premier%20League", "search_league").await;
    let leagues = client(&server)
        .search_leagues("Premier League")
        .await
        .unwrap();
    assert_eq!(leagues.len(), 1);
    let league = &leagues[0];
    assert_eq!(league.id_league.as_deref(), Some("4328"));
    assert_eq!(league.str_league.as_deref(), Some("English Premier League"));
    assert_eq!(
        league.id_league.as_deref().unwrap().parse::<LeagueId>(),
        Ok(LeagueId(4328))
    );
}

#[tokio::test]
async fn search_events_unwraps_envelope() {
    let server = server_with("/search/event/Arsenal", "search_event").await;
    let events = client(&server).search_events("Arsenal").await.unwrap();
    assert!(!events.is_empty());
    assert!(events.iter().all(|e| e.id_event.is_some()));
}

#[tokio::test]
async fn league_lookup_returns_first_record() {
    let server = server_with("/lookup/league/4328", "lookup_league").await;
    let league = client(&server).league(LeagueId(4328)).await.unwrap();
    let league = league.expect("fixture holds one league");
    assert!(league.str_league.is_some());
    assert!(league.str_sport.is_some());
}

#[tokio::test]
async fn event_lookup_returns_first_record() {
    let server = server_with("/lookup/event/441613", "lookup_event").await;
    let event = client(&server).event(EventId(441613)).await.unwrap();
    let event = event.expect("fixture holds one event");
    assert!(event.id_event.is_some());
    assert!(event.str_event.is_some());
    assert!(event.date_event.is_some());
}

#[tokio::test]
async fn seasons_are_flattened_to_labels() {
    let server = server_with("/list/seasons/4328", "list_seasons").await;
    let seasons = client(&server).seasons(LeagueId(4328)).await.unwrap();
    assert!(!seasons.is_empty());
    assert!(seasons.iter().all(|s| !s.is_empty()));
}

#[tokio::test]
async fn season_schedule_rows_carry_event_ids_and_dates() {
    let server = server_with("/schedule/league/4328/2025-2026", "schedule_league_season").await;
    let rows = client(&server)
        .season_schedule(LeagueId(4328), "2025-2026")
        .await
        .unwrap();
    assert!(!rows.is_empty());
    for row in &rows {
        let id = row.id_event.as_deref().expect("every row has idEvent");
        id.parse::<EventId>().expect("idEvent is numeric");
        assert!(row.date_event.is_some());
        assert!(row.str_event.is_some());
    }
}

#[tokio::test]
async fn next_and_previous_events_parse() {
    let server = server_with("/schedule/next/league/4328", "schedule_next_league").await;
    let next = client(&server).next_events(LeagueId(4328)).await.unwrap();
    assert!(!next.is_empty());

    let server = server_with("/schedule/previous/league/4328", "schedule_previous_league").await;
    let previous = client(&server)
        .previous_events(LeagueId(4328))
        .await
        .unwrap();
    assert!(!previous.is_empty());
}

#[tokio::test]
async fn empty_envelope_is_an_empty_list() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/search/league/nothing"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"search":null}"#))
        .mount(&server)
        .await;
    let leagues = client(&server).search_leagues("nothing").await.unwrap();
    assert!(leagues.is_empty());
}

#[tokio::test]
async fn unauthorized_is_an_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
        .mount(&server)
        .await;
    let error = client(&server).seasons(LeagueId(1)).await.unwrap_err();
    assert!(error.is_unauthorized(), "{error}");
    assert!(!error.is_rate_limited());
    assert!(matches!(error, Error::Api { status: 401, .. }));
}

#[tokio::test]
async fn rate_limit_is_reported() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(429).set_body_string("Too Many Requests"))
        .mount(&server)
        .await;
    let error = client(&server).seasons(LeagueId(1)).await.unwrap_err();
    assert!(error.is_rate_limited(), "{error}");
}
