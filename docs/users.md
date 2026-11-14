# Users

## User object

| Field | Type | Notes |
| --- | --- | --- |
| `id` | number | Numeric id |
| `urn` | string | `soundcloud:users:{id}` |
| `uri` | string | API URL |
| `kind` | string | `user` |
| `username` | string | Display name |
| `permalink` | string | Profile slug |
| `permalink_url` | string | Public profile URL |
| `full_name` | string | Display name, can be empty |
| `description` | string | Profile text, can be empty |
| `avatar_url` | string or null | Image URL |
| `city` | string | Can be empty |
| `country_code` | string or null | Country |
| `created_at` | string or null | ISO 8601 |
| `last_modified` | string | ISO 8601 |
| `track_count` | number | Tracks uploaded |
| `playlist_count` | number | Playlists and albums |
| `followers_count` | number | Followers |
| `followings_count` | number | Followed users |
| `likes_count` | number | Likes |
| `reposts_count` | number | Reposts |
| `comments_count` | number | Comments |
| `verified` | boolean | Verification flag |

The measured user object had `created_at: null`. Treat the field as nullable.

## Routes

| Route | Returns |
| --- | --- |
| `GET /users/{user_id}` | One user |
| `GET /users/{user_id}/tracks` | Track objects |
| `GET /users/{user_id}/likes` | Like objects |
| `GET /users/{user_id}/playlists` | Playlist objects, including albums |
| `GET /users/{user_id}/albums` | Playlist objects with `is_album: true` |
| `GET /users/{user_id}/toptracks` | Track objects |
| `GET /users/{user_id}/relatedartists` | User objects |
| `GET /users/{user_id}/followers` | User objects |
| `GET /users/{user_id}/followings` | User objects |

All nine returned 200 with a client id.

## Tracks and top tracks

`/tracks` is paginated. `next_href` carried an opaque offset, not a plain numeric offset. The first page was in reverse creation order.

`/toptracks` returned 30 tracks and `next_href: null`. The order was not creation order. The route returned a fixed first page for the measured user.

## Playlists and albums

`/playlists` includes albums. `/albums` returns only objects with `is_album: true`. In the measured sample every playlist was an album, so both routes returned the same ids.

## Likes

Each item in `/likes` is an envelope, not a bare track:

```json
{
  "created_at": "2025-01-05T00:00:00Z",
  "kind": "like",
  "track": {}
}
```

The `track` value is a track object. The route worked with a client id and no user token.

## Followers, followings and related artists

All three return `collection` arrays of user objects. `next_href` carries the next page. The arrays are not track collections.

## Examples

- [user.json](../examples/user.json)
- [user-tracks.json](../examples/user-tracks.json)
- [user-likes.json](../examples/user-likes.json)
- [user-playlists.json](../examples/user-playlists.json)
- [user-albums.json](../examples/user-albums.json)
- [user-toptracks.json](../examples/user-toptracks.json)
- [user-related-artists.json](../examples/user-related-artists.json)
- [user-followers.json](../examples/user-followers.json)
- [user-followings.json](../examples/user-followings.json)
