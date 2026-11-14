# Search

## Routes

| Route | Returns |
| --- | --- |
| `GET /search/tracks` | Track objects |
| `GET /search/users` | User objects |
| `GET /search/playlists` | Playlist objects |
| `GET /search/albums` | Playlist objects with `is_album: true` |

All four returned 200 with a client id.

## Response

| Field | Meaning |
| --- | --- |
| `collection` | The page of results |
| `total_results` | Total matches |
| `next_href` | Absolute URL for the next page |
| `query_urn` | Search cursor |

`next_href` did not contain `client_id` in the measured responses. Follow it as written.

## Query

| Parameter | Measurement |
| --- | --- |
| `q` | The query string |
| `limit` | Page size |
| `offset` | Numeric page offset |
| `linked_partitioning` | Sent; `next_href` was present with or without it |

`limit=200` returned 200 items. `limit=201` returned 201. The search route does not enforce a 200 item cap.

## Parameters with no measured effect

These parameters returned the same total and the same first track as a request without them:

| Parameter | Value sent |
| --- | --- |
| `filter` | `streamable` |
| `license` | `cc-by` |
| `duration` | `short` |
| `created_at` | `last_week` |
| `order` | `latest` |

The service accepted the parameters and ignored them. The catalogue does not list them as filters.

## Track results

Search track items are full track objects. They carried `media.transcodings[]` in this measurement. The same track fields appear as on `/tracks/{track_id}`.

## Examples

- [search-tracks.json](../examples/search-tracks.json)
- [search-users.json](../examples/search-users.json)
- [search-playlists.json](../examples/search-playlists.json)
- [search-albums.json](../examples/search-albums.json)
