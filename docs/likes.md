# Likes

## Write routes

| Method | Path | Auth |
| --- | --- | --- |
| `PUT` | `/users/{user_id}/track_likes/{track_id}` | User token |
| `DELETE` | `/users/{user_id}/track_likes/{track_id}` | User token |

The path carries the user id and the track id. A client id alone is not enough. A request with browser headers, a client id and no `Authorization` header returned 401 with an empty body. It did not return 400.

That 401 is an auth challenge. It is not a validation error, and it is not evidence about the track or user id. No like was created or removed.

## Read route

`GET /users/{user_id}/likes` returned 200 with a client id and no user token. Each item is a `like` object with a `track` object.

```json
{
  "created_at": "2025-01-05T00:00:00Z",
  "kind": "like",
  "track": {}
}
```

## What a client does

A like is a `PUT` to the user and track path. An unlike is a `DELETE` to the same path. Both requests need `Authorization: OAuth <token>`.

The successful write response was not measured. Do not infer a body from the 401.

## Examples

- [user-likes.json](../examples/user-likes.json)
