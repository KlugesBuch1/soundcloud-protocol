# Client guide

What a player has to get right beyond the individual routes. The measurements
behind each point are in the linked pages.

## Client id lifetime

Resolve the id once per app session and keep it in memory:

1. `GET https://soundcloud.com/` and collect the `a-v2.sndcdn.com/assets/*.js` URLs.
2. Scan those assets for the `client_id:"<32 characters>"` literal.
3. Hold the value for the lifetime of the process.

Do not resolve it per track, per stream or per resolve call. Discovery costs one
HTML request plus one request per referenced asset, and it is the part of the
flow most likely to be rate limited. A player that resolves per play multiplies
that cost by every track change.

Writing it to disk buys nothing: it rotates, so a stored id eventually returns
`401` on every call. Resolve it at start instead.

### Refresh on 401

The id changing is only visible through a `401`. Handle it in one place:

| Step | Action |
| --- | --- |
| 1 | Send the request with the held id. |
| 2 | On `401`, resolve the id again. |
| 3 | If the new id differs, retry the request once. |
| 4 | If it is unchanged, the `401` is an auth gate, not a stale id. Surface it. |

Collapse concurrent refreshes into one discovery. Without that, ten parallel
requests that all hit `401` start ten discoveries and make the problem worse.

There is no useful TTL. A refresh costs a discovery, and a discovery that is not
needed is pure overhead; the `401` is the signal.

`soundcloud-v2` implements this: `Client::discover()` resolves once,
`Client::refresh_client_id()` forces a refresh, and every request retries once
after a `401` if the id changed.

## Playback loop

1. Read the track and keep `media.transcodings[]`.
2. Pick a transcoding (below).
3. Resolve it immediately before playback.
4. Play the returned URL.
5. On a CDN `403` or `404`, resolve again once.

The resolved URL is signed and short lived: about 389 seconds for progressive
and 7,331 seconds for HLS ([streams.md](streams.md)). Resolve at play time, not
when the queue is built. A queue that resolves ten tracks up front has stale
URLs by the time the tenth one starts.

For progressive streams, seek with HTTP range requests. For HLS, hand the
playlist URL to the player.

## Choosing a transcoding

Walk the list in this order and stop at the first playable entry:

| Priority | Condition | Why |
| --- | --- | --- |
| 1 | `format.protocol == "progressive"` and not `snipped` | One file, range seek, no playlist handling |
| 2 | `format.protocol == "hls"` and not `snipped` | Segmented playback, longer URL lifetime |
| — | `snipped == true` | 30 second preview; label it, do not present it as the track |
| — | `ctr-encrypted-hls` / `cbc-encrypted-hls` | Needs a licence exchange this catalogue does not perform |
| — | empty `transcodings` | `BLOCK`; metadata only |

`policy` is `ALLOW`, `MONETIZE`, `SNIP` or `BLOCK` ([tracks.md](tracks.md)).
`MONETIZE` still carries transcodings; monetization is not a playback gate in
the measured objects. This catalogue does not measure ad delivery, so it makes
no claim about what a player should do with ads.

When only encrypted protocols are present and every clear protocol returns
`404`, treat the track as not playable rather than retrying.

## Pagination

`next_href` is absolute and already carries the cursor.

- On search routes it does **not** contain `client_id`; append it or the next
  request returns `401` ([search.md](search.md)).
- Search pages use a numeric `offset`; user lists use an opaque cursor in the
  same parameter. Follow `next_href` instead of constructing either.
- `limit=200` and `limit=201` both worked. For a UI, 50 to 100 keeps the first
  paint fast.
- Deduplicate by `id` when merging pages; tracks can repeat across a boundary.

`soundcloud-v2` exposes `Client::next_page(&page)` for this.

## Caching

| Data | Keep | Do not |
| --- | --- | --- |
| Client id | In memory for the session | Persist across runs |
| Track and user metadata | In memory, short TTL | Persist a full library |
| Stream URLs | Only for the current play | Reuse on the next play |
| Artwork | Normal HTTP cache | Request every size for every row |

The catalogue has no route for audio caching or offline downloads. A signed
stream URL is valid for one play anyway; anything kept longer needs its own
invalidation.

## Errors and retries

| Status | Meaning | Action |
| --- | --- | --- |
| `401` | Stale client id or missing user token | Refresh the id once, then surface |
| `404` with `{}` | Object does not exist | Do not retry |
| `404` with an empty body | Path does not exist | Do not retry |
| `429` | Not observed | Back off; concurrency cap of a few requests is enough |
| `5xx` | Service side | Retry idempotent `GET`s with jittered backoff |

No rate limit headers were observed ([errors.md](errors.md)). Cap concurrency
rather than relying on a header.

## UI details

- `artwork_url` can be null. Fall back to the user's `avatar_url`, then to a
  placeholder. Swap the `-large` suffix for `-t500x500` or `-t200x200`
  ([tracks.md](tracks.md)).
- Show `full_duration` when `policy` is `SNIP`, and mark the track as a preview.
- `release_date` and `display_date` are often null; do not sort on them blindly.
- `streamable == false` or no transcodings means the row is metadata only.

## Not covered yet

- **User token flow.** `/me`, likes, comments and own playlists need an
  `Authorization: OAuth <token>` header. The catalogue records the 401 but not
  how to obtain or refresh the token ([auth.md](auth.md)).
- **Ads.** No ad route or ad segment was measured.
- **Rate limits.** No `429` and no rate limit headers were observed.
- **Search filters.** `filter`, `license`, `duration`, `created_at` and `order`
  were accepted and ignored ([search.md](search.md)).
