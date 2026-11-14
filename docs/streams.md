# Streams

## Two-step resolve

| Step | Request | Result |
| --- | --- | --- |
| 1 | `GET /tracks/{track_id}` | `media.transcodings[]` |
| 2 | `GET /media/{media_path}` | JSON with `url` |
| 3 | `GET url` | Audio bytes or an HLS playlist |

The transcoding `url` is an `api-v2.soundcloud.com` URL. The part after `/media/` is the `{media_path}` value. The `client_id` is required on step 2.

## Protocols

| `format.protocol` | Shape | Measurement |
| --- | --- | --- |
| `progressive` | One audio file | Range request returned 206 with `Content-Type: audio/mpeg` |
| `hls` | Playlist of segments | 200 with `application/vnd.apple.mpegurl`; the measured playlist had 11 segments |
| `ctr-encrypted-hls` | Encrypted HLS | Resolve returned a licence token; the playlist had `#EXT-X-KEY` |
| `cbc-encrypted-hls` | Encrypted HLS | Resolve returned a licence token |

The encrypted protocols appeared on licensed content. The playlist refers to a key, and the resolve response carries a licence token. This catalogue does not perform the licence exchange.

A track with only encrypted transcodings returned 404 with `{}` for `progressive` and `hls` requests on the same media id. A 404 on every clear protocol is evidence of DRM when the track object exposes encrypted protocols.

## Previews

A track with `policy: SNIP` returned `duration: 30000` and `full_duration: 268929`. Both transcodings had `snipped: true` and `duration: 30000`. A preview is 30 seconds.

The truncated duration is on the transcoding and on the track's `duration`. `full_duration` remains the length of the complete track.

## Expiry

The resolved `url` is signed and time-limited.

| Resolve | Time to expiry |
| --- | --- |
| Progressive | 389 seconds |
| HLS | 7,331 seconds |

The CDN can reject a stale URL. Resolve the transcoding again before every play. Do not cache the signed URL for a later session.

## What a client does

1. Read the track and choose a transcoding.
2. Resolve that transcoding immediately before playback.
3. Play the returned URL.
4. Resolve again for the next attempt.

For encrypted HLS, complete the licence exchange before loading segments. For progressive, seek with HTTP range requests.

## Examples

- [media-progressive.json](../examples/media-progressive.json)
- [media-hls.json](../examples/media-hls.json)
- [media-encrypted.json](../examples/media-encrypted.json)
