# Conventions

## Route notation

Paths use braces for values:

```text
/tracks/{track_id}
/users/{user_id}/track_likes/{track_id}
/media/{media_path}
```

| Placeholder | Value |
| --- | --- |
| `{track_id}` | Numeric track id |
| `{track_ids}` | Comma-separated track ids |
| `{user_id}` | Numeric user id |
| `{playlist_id}` | Numeric playlist id |
| `{media_path}` | The path after `/media/` in a transcoding `url` |

Example ids in this repository are synthetic. Track `123456789`, user `987654321` and playlist `246810121` do not identify live objects.

## Headers

| Header | Value | When |
| --- | --- | --- |
| `User-Agent` | Desktop Chrome string | Every measured request |
| `Accept` | `application/json, text/plain, */*` | Every measured request |
| `Accept-Language` | `en-US,en;q=0.9` | Every measured request |
| `Origin` | `https://soundcloud.com` | Every measured request |
| `Referer` | `https://soundcloud.com/` | Every measured request |
| `Authorization` | `OAuth <token>` | Routes that need a user token |

`client_id` is a query parameter, not a header. A request with `User-Agent` and `Accept` and no `Origin` or `Referer` returned 200 for `/tracks/{track_id}`. The header list records what the requests carried. It does not show that every header is required. The `auth` field in `endpoints.yaml` names the credential the route needs. The write routes and `/me` were measured without a user token; their 401 came from an unauthenticated request. Query parameters have the same limit: a listed parameter was sent, and the notes record the cases where it was required.

## Status codes

| Status | Body | What it proved |
| --- | --- | --- |
| 200 | JSON | The route and the object existed |
| 206 | Audio bytes | A range request for a progressive stream worked |
| 400 | JSON with `errors` | The route exists and a parameter was missing |
| 401 | Empty | An auth gate was reached |
| 404 | Empty or `{}` | The path or the object was not found |
| 429 | Not observed | No rate limit response was measured |

A 401 with a missing client id does not prove that the path exists. A 404 with a missing object does not prove that the path does not exist. The body shape separates some 404s. An unknown path returned an empty body. A missing track returned `{}`.

## urn and id

| Field | Example | Use |
| --- | --- | --- |
| `id` | `123456789` | Numeric id in paths and queries |
| `urn` | `soundcloud:tracks:123456789` | URN in JSON |
| `uri` | `https://api.soundcloud.com/tracks/soundcloud%3Atracks%3A123456789` | API URL with the URN encoded |

`id` is a number. `urn` is a string with a kind and the same number. A path takes the number, not the URN.

## Pagination

List routes return an object with `collection`. `/tracks?ids=` is the exception: it returns a bare array. Track and user routes also return `next_href`. Search routes add `total_results` and `query_urn`.

| Field | Meaning |
| --- | --- |
| `collection` | The page of objects |
| `next_href` | Absolute URL for the next page, or null |
| `total_results` | Search total, not the page size |
| `query_urn` | Search or chart cursor used in `next_href` |

`limit=200` returned 200 items on `/search/tracks`. `limit=201` returned 201. The 200 value is a page size, not a hard cap on that route. Follow `next_href`. Search pages carry a numeric `offset`; user lists carry an opaque cursor in the same parameter.

## Time

Object timestamps are UTC ISO 8601 strings with a `Z` suffix and second precision, for example `2025-01-01T00:00:00Z`. The `next_href` cursor for user tracks can contain milliseconds. Chart `last_updated` uses the same timestamp format.
