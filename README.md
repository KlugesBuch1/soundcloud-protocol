# SoundCloud protocol

[![CI](https://github.com/KlugesBuch1/soundcloud-protocol/actions/workflows/checks.yml/badge.svg)](https://github.com/KlugesBuch1/soundcloud-protocol/actions/workflows/checks.yml)
[![crates.io](https://img.shields.io/crates/v/soundcloud-v2.svg)](https://crates.io/crates/soundcloud-v2)

A catalogue of the HTTP API used by the SoundCloud web player. It covers `api-v2.soundcloud.com`, from tracks and streams to search, users, playlists, charts, comments and likes.

It is unofficial and does not cover the whole API surface.

## This repo contains

| Path | What it is |
| --- | --- |
| [`endpoints.yaml`](endpoints.yaml) | All 27 measured routes |
| [`docs/conventions.md`](docs/conventions.md) | Placeholders, headers, pagination and status codes |
| [`docs/auth.md`](docs/auth.md) | The client id, the user token and the routes that need one |
| [`docs/tracks.md`](docs/tracks.md) | The track object, transcodings, policies and comments |
| [`docs/streams.md`](docs/streams.md) | The two-step resolve and the four protocols |
| [`docs/search.md`](docs/search.md) | Tracks, users, playlists and albums |
| [`docs/users.md`](docs/users.md) | The user object, track lists and top tracks |
| [`docs/playlists.md`](docs/playlists.md) | Sets, albums and `track_count` |
| [`docs/charts.md`](docs/charts.md) | Kinds, genres and the score envelope |
| [`docs/likes.md`](docs/likes.md) | The like and unlike writes |
| [`docs/errors.md`](docs/errors.md) | 401, 404 and the shape each one returns |
| [`docs/endpoint-index.md`](docs/endpoint-index.md) | The same 27 routes as a table |
| [`tools/probe.py`](tools/probe.py) | Re-checks every read route and prints the status |
| [`tools/from_har.py`](tools/from_har.py) | Extracts routes from a HAR file and writes trimmed examples |
| [`examples/`](examples) | Trimmed response bodies with placeholder ids |

## Rust client

[`crates/soundcloud-v2`](crates/soundcloud-v2) is the reference Rust client for these endpoints. It
covers tracks, tracks by ids, search, users, playlists, resolve, transcoding resolve, charts and
comments. Published on crates.io as [`soundcloud-v2`](https://crates.io/crates/soundcloud-v2).

## The index

Each entry in `endpoints.yaml` looks like this:

```yaml
- id: track
  area: tracks
  method: GET
  host: api-v2.soundcloud.com
  path: /tracks/{track_id}
  auth: client_id
  class: read
  status: 200
  doc: docs/tracks.md
  summary: One track and its transcodings.
  query:
    - client_id
  notes: "media.transcodings[] carried HLS entries. A second measured track carried HLS and progressive entries."
```

`status` is what the route returned when it was measured. A 401 means the path exists and the service wants a user token. A 404 is either a dead path or a missing object.

`class` is `read` or `write`. The three write routes and `/me` show 401 because no user token was sent.

`auth` names the credential the route needs. `client_id` is a query parameter. `user-token` is the `Authorization` header. `query` lists the parameters that were sent, and `notes` records what the measurement showed.

## Auth explained

The player sends a client id on every route:

```http
GET /tracks/{track_id}?client_id={client_id}
```

The id is a 32-character literal in a versioned asset under `a-v2.sndcdn.com/assets/`. The asset name changes and the id rotates. It is checked before the path: a request without a valid id returns 401 with an empty body, even for a path that does not exist.

Routes that act on an account take a second credential:

```http
Authorization: OAuth <token>
```

`/me`, `POST /tracks/{track_id}/comments` and both like routes returned 401 without it. The catalogue does not record how to obtain a token. The full list of routes and the measured responses are in [`docs/auth.md`](docs/auth.md).

## Read a route before you call it

The docs use placeholders:

- `{track_id}` is the numeric track id.
- `{track_ids}` is a comma-separated list of track ids.
- `{user_id}` is the numeric user id.
- `{playlist_id}` is the numeric playlist id.
- `{media_path}` is the path after `/media/` in a transcoding `url`.

`id` is a number and `urn` is a string with the same number: `123456789` against `soundcloud:tracks:123456789`. A path takes the number.

All ids in the docs and examples are placeholders. The examples use track `123456789`, user `987654321`, playlist `246810121`, `Contoso` and `Example Artist`. The repository contains no real credential or account data.

## Probe

`tools/probe.py` reads `endpoints.yaml` and sends every read route with browser headers and at least one second between requests. It sends `{}` for reads that are POST. It needs a client id in `SC_CLIENT_ID`, never resolves one itself and never writes one to disk.

```powershell
python -m pip install pyyaml
$env:SC_CLIENT_ID = Read-Host
python tools/probe.py
```

It prints one line per route with the expected and actual status and exits 1 on a mismatch. `--update` rewrites only the `status` value of the affected entry.

## What is not here

- Uploads and account settings
- Messaging and payments
- Private resources
- The DRM licence exchange
- CDN media serving
- Likes, comments and follows that were actually performed

No route was dropped for a 404. The three write routes and `/me` were measured without a user token, so 401 is the only status recorded for them. No 429 was observed.

## Accuracy

The service changes. A route that once returned 200 can disappear, and the client id rotates.

1. Resolve a current client id before running the probe.
2. Check the route with `tools/probe.py` before you trust it.
3. Treat headers as observed, not required. The list records what the measured requests carried.

Last check: 2026-11-14
