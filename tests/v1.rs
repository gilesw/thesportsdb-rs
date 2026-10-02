//! Fixtures are free-key (`123`) v1 responses for the UFC league (4443),
//! captured on 2026-10-02.

use thesportsdb::v1::{Client, FREE_KEY};
use thesportsdb::{Error, LeagueId};
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const UFC: LeagueId = LeagueId(4443);

fn fixture(name: &str) -> String {
    let file = format!(
        "{}/tests/fixtures/v1/{name}.json",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&file).unwrap_or_else(|e| panic!("{file}: {e}"))
}

async fn server_with(route: &str, params: &[(&str, &str)], name: &str) -> MockServer {
    let server = MockServer::start().await;
    let mut mock = Mock::given(method("GET")).and(path(format!("/api/v1/json/{FREE_KEY}{route}")));
    for (key, value) in params {
        mock = mock.and(query_param(*key, *value));
    }
    mock.respond_with(ResponseTemplate::new(200).set_body_string(fixture(name)))
        .expect(1)
        .mount(&server)
        .await;
    server
}

fn client(server: &MockServer) -> Client {
    Client::with_base_url(format!("{}/api/v1/json/{FREE_KEY}", server.uri()))
}

#[tokio::test]
async fn league_lookup_returns_ufc() {
    let server = server_with("/lookupleague.php", &[("id", "4443")], "lookup_league").await;
    let league = client(&server).league(UFC).await.unwrap().unwrap();
    assert_eq!(league.id_league.as_deref(), Some("4443"));
    assert_eq!(league.str_league.as_deref(), Some("UFC"));
    assert_eq!(league.str_sport.as_deref(), Some("Fighting"));
    assert_eq!(league.str_current_season.as_deref(), Some("2026"));
}

#[tokio::test]
async fn seasons_are_flattened_to_labels() {
    let server = server_with("/search_all_seasons.php", &[("id", "4443")], "seasons").await;
    let seasons = client(&server).seasons(UFC).await.unwrap();
    assert_eq!(seasons.first().map(String::as_str), Some("1993"));
    assert!(seasons.iter().all(|s| s.len() == 4));
}

#[tokio::test]
async fn season_events_carry_ids_titles_and_dates() {
    let server = server_with(
        "/eventsseason.php",
        &[("id", "4443"), ("s", "2026")],
        "season_events",
    )
    .await;
    let events = client(&server).season_events(UFC, "2026").await.unwrap();
    assert!(!events.is_empty());
    for event in &events {
        event.id_event.as_deref().unwrap().parse::<i64>().unwrap();
        assert!(event.str_event.as_deref().unwrap().starts_with("UFC"));
        assert!(event.date_event.flatten().is_some());
    }
}

#[tokio::test]
async fn next_and_previous_events_parse() {
    let server = server_with("/eventsnextleague.php", &[("id", "4443")], "next_events").await;
    let next = client(&server).next_events(UFC).await.unwrap();
    assert_eq!(next.len(), 1);

    let server = server_with("/eventspastleague.php", &[("id", "4443")], "past_events").await;
    let previous = client(&server).previous_events(UFC).await.unwrap();
    assert_eq!(previous.len(), 1);
}

#[tokio::test]
async fn empty_envelope_is_an_empty_list() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"events":null}"#))
        .mount(&server)
        .await;
    assert!(client(&server).next_events(UFC).await.unwrap().is_empty());
}

#[tokio::test]
async fn rate_limit_is_reported() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(429).set_body_string("Too Many Requests"))
        .mount(&server)
        .await;
    let error = client(&server).next_events(UFC).await.unwrap_err();
    assert!(error.is_rate_limited(), "{error}");
    assert!(matches!(error, Error::Api { status: 429, .. }));
}

#[test]
fn unusable_keys_fall_back_to_the_free_key() {
    for key in ["", "  ", "a/b", "x?y"] {
        assert_eq!(format!("{:?}", Client::new(key)), "Client { .. }");
    }
}
