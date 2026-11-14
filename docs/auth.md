# Authentication

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

The catalogue does not record how to obtain a user token. That flow was not measured.
