# Playlists

## Playlist object

A set and an album are both `kind: playlist`. `is_album` separates them.

| Field | Type | Notes |
| --- | --- | --- |
| `id` | number | Numeric id |
| `kind` | string | `playlist` |
| `is_album` | boolean | True for an album |
| `set_type` | string | `""` for a set, `ep` for the measured album |
| `track_count` | number | Total tracks |
| `tracks` | array | Track objects |
| `title` | string | Display title |
| `permalink_url` | string | Public web URL |
| `artwork_url` | string or null | Image URL |
| `created_at` | string | ISO 8601 |
| `release_date` | string or null | ISO 8601 |
| `published_at` | string | ISO 8601 |
| `display_date` | string | ISO 8601 |
| `duration` | number | Milliseconds |
| `genre` | string | Can be empty |
| `label_name` | string | Can be empty |
| `user_id` | number | Owner id |
| `user` | object | User object |

`track_count` is the total, not the length of the current `tracks` page.

## Tracks field

`tracks[]` was present on the measured playlist and album responses. The 33 track playlist had `track_count: 33` and 33 entries. Search playlist results also carried `tracks[]` in this measurement. The catalogue does not claim that every list response includes the array.

## Routes

| Route | Result |
| --- | --- |
| `GET /playlists/{playlist_id}` | One playlist or album |
| `GET /users/{user_id}/playlists` | Sets and albums |
| `GET /users/{user_id}/albums` | Albums only |
| `GET /tracks/{track_id}/albums` | Albums containing the track |
| `GET /tracks/{track_id}/playlists_without_albums` | Sets containing the track |

For the measured user, all three playlists had `is_album: true`. Both user routes returned the same three ids. The albums route is the filtered view; the playlists route can contain both.

The track albums route returned an empty `collection` for the measured track. The playlists-without-albums route returned nine items.

## Examples

- [playlist.json](../examples/playlist.json)
- [album.json](../examples/album.json)
- [user-playlists.json](../examples/user-playlists.json)
- [user-albums.json](../examples/user-albums.json)
- [track-albums.json](../examples/track-albums.json)
- [track-playlists-without-albums.json](../examples/track-playlists-without-albums.json)
