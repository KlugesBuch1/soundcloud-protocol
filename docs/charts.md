# Charts

## Query

| Parameter | Value | Result |
| --- | --- | --- |
| `kind` | `trending` | 200 |
| `kind` | `top` | 404 with `{}` |
| `kind` | `new` | 400 with `{}` |
| `genre` | `soundcloud:genres:all-music` | 200 |
| `genre` | `soundcloud:genres:electronic` | 404 with `{}` |
| `genre` | `soundcloud:genres:rock` | 404 with `{}` |
| `genre` | `soundcloud:genres:hiphoprap` | 404 with `{}` |
| `genre` | `soundcloud:genres:notreal` | 404 with `{}` |
| `limit` | Number | Page size |
| `offset` | Number | Page offset |

The working pair in this measurement was `kind=trending` with `genre=soundcloud:genres:all-music`. `kind=top` did not resolve.

## Response

| Field | Meaning |
| --- | --- |
| `kind` | Echoes the requested kind |
| `genre` | Echoes the requested genre |
| `last_updated` | ISO 8601 timestamp |
| `collection` | `{score, track}` entries |
| `query_urn` | Chart cursor |
| `next_href` | Next page, or null |

`score` is a float, not a percentage. The first three scores measured were `0.8455197261914359`, `0.7693791946797635` and `0.7422920348954167`.

Each entry has a track object under `track`. The entry itself is not a track.

## Examples

- [charts.json](../examples/charts.json)
