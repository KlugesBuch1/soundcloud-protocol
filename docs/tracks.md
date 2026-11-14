# Tracks

## Track object

| Field | Type | Notes |
| --- | --- | --- |
| `id` | number | Numeric id |
| `urn` | string | `soundcloud:tracks:{id}` |
| `uri` | string | API URL with the encoded URN |
| `kind` | string | `track` |
| `title` | string | Display title |
| `permalink_url` | string | Public web URL |
| `artwork_url` | string or null | Image URL |
| `created_at` | string | ISO 8601 |
| `last_modified` | string | ISO 8601 |
| `release_date` | string or null | ISO 8601 |
| `duration` | number | Milliseconds, shortened for a preview |
| `full_duration` | number | Milliseconds, full track |
| `streamable` | boolean | Stream flag |
| `policy` | string | Playback policy |
| `monetization_model` | string | `NOT_APPLICABLE`, `AD_SUPPORTED`, `SUB_HIGH_TIER` or `BLACKBOX` |
| `commentable` | boolean | Comment flag |
| `comment_count` | number | Total comments |
| `likes_count` | number | Total likes |
| `playback_count` | number | Total plays |
| `reposts_count` | number | Total reposts |
| `user_id` | number | Owner id |
| `user` | object | User object |
| `media` | object | Container for `transcodings[]` |

`duration`, `full_duration`, comment timestamps and transcoding durations are milliseconds.

## Policy

| Value | Meaning | Measurement |
| --- | --- | --- |
| `ALLOW` | Playable | HLS transcodings, no ad model |
| `MONETIZE` | Ad or subscription content | HLS and progressive transcodings |
| `SNIP` | 30 second preview | `duration: 30000`, `full_duration: 268929` |
| `BLOCK` | Metadata only | 200, `media.transcodings: []` |

`access` did not appear in any measured track object. That includes `ALLOW`, `MONETIZE`, `SNIP` and `BLOCK` tracks. Playback state is carried by `policy`, `streamable` and `media.transcodings[]`.

## Transcodings

| Field | Meaning |
| --- | --- |
| `url` | `api-v2.soundcloud.com` media URL |
| `preset` | Encoding preset, for example `aac_160k` or `mp3_0_0` |
| `duration` | Milliseconds for this rendition |
| `snipped` | True on a preview rendition |
| `quality` | Quality label, for example `sq` |
| `format.protocol` | `progressive`, `hls`, `ctr-encrypted-hls` or `cbc-encrypted-hls` |
| `format.mime_type` | MIME type of the rendition |

A `SNIP` track carried `snipped: true` and `duration: 30000` on both of its transcodings. A `BLOCK` track carried an empty array.

## Artwork sizes

The suffix is part of the image URL. Replace `-large` with another suffix.

| Suffix | Pixels |
| --- | --- |
| `-large` | 100 x 100 |
| `-t500x500` | 500 x 500 |
| `-t300x300` | 300 x 300 |
| `-t200x200` | 200 x 200 |
| `-t120x120` | 120 x 120 |
| `-t67x67` | 67 x 67 |
| `-mini` | 16 x 16 |
| `-tiny` | 20 x 20 |
| `-small` | 32 x 32 |
| `-crop` | 225 x 300 |
| `-badge` | 47 x 47 |

The measured `-large` image was 100 x 100, not 500 x 500.

## Track ids

| Route | Result |
| --- | --- |
| `GET /tracks?ids={track_ids}` | A `collection` of tracks |
| `GET /tracks/{track_id}` | One track object |

`{track_ids}` is comma-separated. The ids route returns a single response, not a paginated page.

## Comments

| Parameter | Use |
| --- | --- |
| `threaded` | Required. The player sends `0` or `1` |
| `limit` | Page size |
| `offset` | Cursor |

A request without `threaded` returned 400:

```json
{"errors":["Did not get or could not parse multiple missing params"]}
```

| Comment field | Type | Notes |
| --- | --- | --- |
| `id` | number | Numeric id |
| `kind` | string | `comment` |
| `body` | string | Comment text |
| `created_at` | string | ISO 8601 |
| `timestamp` | number | Milliseconds into the track |
| `track_id` | number | Track id |
| `user_id` | number | Author id |
| `user` | object | Author object |
| `self.urn` | string | `soundcloud:comments:{id}` |

A comment write uses `POST /tracks/{track_id}/comments` with `body` and `timestamp` as query parameters. It needs a user token. An unauthenticated request returned 401 with an empty body. No comment was created.

## Related collections

| Route | Result | Measurement |
| --- | --- | --- |
| `GET /tracks/{track_id}/related` | Tracks | 200 |
| `GET /tracks/{track_id}/albums` | Albums | 200, empty collection |
| `GET /tracks/{track_id}/playlists_without_albums` | Playlists | 200, nine items |

## Resolve

`GET /resolve` takes a public URL in `url`. A track URL returned a track object. A playlist URL returned a playlist object. The response is the object itself, not a `collection`.

## Search results

`/search/tracks` returned track objects with `media.transcodings[]` in this measurement. The catalogue does not assume that `media` is absent from search results.

## Examples

- [track.json](../examples/track.json)
- [track-snipped.json](../examples/track-snipped.json)
- [track-blocked.json](../examples/track-blocked.json)
- [track-encrypted.json](../examples/track-encrypted.json)
- [tracks.json](../examples/tracks.json)
- [comments.json](../examples/comments.json)
- [track-related.json](../examples/track-related.json)
- [track-albums.json](../examples/track-albums.json)
- [track-playlists-without-albums.json](../examples/track-playlists-without-albums.json)
