# Authentication

## Which credential where

`client_id` is required on every measured route, `/media/{media_path}` included. It identifies the player, not the user.

A user token is a second header, and only account-scoped routes need it:

| Route | `client_id` | User token |
| --- | --- | --- |
| Metadata: `/tracks`, `/search`, `/users/{id}`, `/playlists`, `/charts`, `/resolve` | required | not needed |
| `/media/{media_path}` | required | not needed |
| `/me` | required | required |
| Comment and like writes | required | required |

Browsing, search and playback work with a `client_id` alone. The user token comes in when an action belongs to an account.

## Client id

| Item | Value |
| --- | --- |
| Parameter | `client_id` |
| Position | Query string |
| Source | A versioned asset under `a-v2.sndcdn.com/assets/` |
| Lifetime | Rotates |

The web player loads a JavaScript asset and reads the id from a `client_id:"<32 characters>"` literal. The asset name changes. Treat the value as a credential and resolve the current one for each session.

The client id reaches the service before any path lookup. Requests without a valid value return 401 with a zero-byte body.

| Request | Status | Body |
| --- | --- | --- |
| `GET /tracks/{track_id}` without `client_id` | 401 | 0 bytes |
| `GET /this/path/does/not/exist` without `client_id` | 401 | 0 bytes |

The two responses are identical. A tokenless probe cannot separate a live route from a dead one. The path is checked after the client id.

A stored id can go stale. Resolve the current asset when a 401 appears.

`tools/probe.py` and `Client::discover()` resolve the value at runtime. They read the landing page, collect the assets it references and search each one for the literal. `SC_CLIENT_ID` overrides the result in the probe. The value stays in memory. If the page or the literal changes shape, the resolver returns an error instead of sending a stale value.

## Lifetime in a client

Resolve once per session, keep the value in memory, and refresh it when a request returns `401`. Do not resolve per track, per stream or per play: discovery is one HTML request plus one request per referenced asset, and repeating it is the fastest way to get the flow rate limited.

| Event | Action |
| --- | --- |
| App start | Resolve once |
| `401` | Resolve again; retry the request once if the id changed |
| Same id after resolve | The `401` is a user-token gate, not a stale id |
| App exit | Drop the value; write nothing to disk |

A stored id goes stale because the value rotates, and no TTL predicts when. The `401` is the only reliable signal. Concurrent refreshes should collapse into one discovery.

`Client::discover()` resolves once, `Client::refresh_client_id()` forces a refresh, and the request path retries once after a `401` when the id changed. See [client.md](client.md) for the playback and refresh loop.

## User token

The user credential is a header:

```http
Authorization: OAuth <token>
```

`/me` with a client id and no user token returned 401 with an empty body. Write routes behaved the same.

| Route | Client id only | User token |
| --- | --- | --- |
| `GET /me` | 401 | Required |
| `POST /tracks/{track_id}/comments` | 401 | Required |
| `PUT /users/{user_id}/track_likes/{track_id}` | 401 | Required |
| `DELETE /users/{user_id}/track_likes/{track_id}` | 401 | Required |
| `GET /users/{user_id}/likes` | 200 | Not required |
| `GET /users/{user_id}/followers` | 200 | Not required |
| `GET /users/{user_id}/followings` | 200 | Not required |

The catalogue does not record how to obtain a user token. That flow was not measured. A client that only browses and plays does not need a user token; [client.md](client.md) lists what does.
