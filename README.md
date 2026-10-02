# thesportsdb

Unofficial typed async Rust client for the
[TheSportsDB](https://www.thesportsdb.com) JSON API, versions 1 and 2.

The low-level clients and types are generated from TheSportsDB's published
OpenAPI documents with [openapi-to-rust](https://github.com/gpu-cli/openapi-to-rust).
A thin hand-written layer per version adds the API key, `LeagueId` and
`EventId` newtypes, and methods that unwrap the response envelopes.

This crate is not affiliated with TheSportsDB. Its data is crowd-sourced and
funded by supporters; a [paid key](https://www.thesportsdb.com/pricing)
unlocks v2 and full v1 lists.

```toml
[dependencies]
thesportsdb = "0.1"
```

## Usage

Each API version is a module with its own `Client`. `LeagueId`, `EventId`
and `Error` are shared.

```rust
use thesportsdb::{LeagueId, v1, v2};

let ufc = LeagueId(4443);

let free = v1::Client::new(v1::FREE_KEY);
for event in free.next_events(ufc).await? {
    println!("{:?} {:?}", event.date_event, event.str_event);
}

let paid = v2::Client::new(std::env::var("THESPORTSDB_API_KEY")?);
for league in paid.search_leagues("UFC").await? {
    println!("{:?}", league.str_league);
}
```

- **v1** carries the key in the URL path. The public free key `123` works but
  caps every list, for example to five seasons, the first five events of a
  season, and one next and one past event.
- **v2** sends the key in the `X-API-KEY` header and needs a paid key. It
  rejects `123` with HTTP 400, "Invalid Premium API key".

`Error::is_unauthorized` and `Error::is_rate_limited` pick out the two
responses a caller usually handles.

## Layout

- `specs/thesportsdb-v1.yaml`, `specs/thesportsdb-v2.yaml`: the OpenAPI
  documents, copied from `https://www.thesportsdb.com/api/spec/<version>/openapi.yaml`.
- `specs/v1.overlay.yaml`, `specs/v2.overlay.yaml`: OpenAPI Overlays that
  correct each schema where the API disagrees with it. See below.
- `openapi-to-rust.v1.toml`, `openapi-to-rust.v2.toml`: generator configuration.
- `src/v1/generated/`, `src/v2/generated/`: generator output, skipped by
  rustfmt and never edited by hand. `effective.json` is the document after
  the overlay. The generator's `mod.rs` there is not used; `src/v1.rs` and
  `src/v2.rs` declare the generated files themselves.
- `src/v1/client.rs`, `src/v2/client.rs`, `src/error.rs`, `src/ids.rs`: the
  hand-written layer.
- `tests/fixtures/v1/`: free-key responses for the UFC league, captured on
  2026-10-02. `tests/fixtures/v2/`: the v2 document's response examples, since
  v2 cannot be captured without a paid key.

## Schema quirks

Both versions' responses disagree with their schemas, so each overlay loosens
three things before generation:

- Timestamps such as `"2014-12-29T20:00:00"` have no zone and are not
  RFC 3339. `date-time` formats are dropped, so they are plain strings.
- Every number is sent as a string, for example `"intRound": "19"`.
  Response schema integers become strings.
- Missing links are sent as `""`. `uri` and `url` formats are dropped, so they are
  plain strings.

Dates such as `dateEvent` are well-formed and stay `chrono::NaiveDate`.

## Regenerating

The generator version is pinned in `mise.toml`.

```sh
for v in v1 v2; do
  curl -sSL -o specs/thesportsdb-$v.yaml https://www.thesportsdb.com/api/spec/$v/openapi.yaml
done
mise run generate
mise run check
```

`mise run generate:check` fails when the committed output is stale.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
