# soundcloud-v2

Read-only Rust client for the `api-v2.soundcloud.com` routes in the catalogue: tracks, related
tracks, albums, search, users, user collections, playlists, resolve, transcoding resolve, charts and
comments.

Unofficial. Not affiliated with SoundCloud.

## Install

```toml
[dependencies]
soundcloud-v2 = "0.2"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## Use

```rust,no_run
use soundcloud_v2::Client;

#[tokio::main]
async fn main() -> Result<(), soundcloud_v2::Error> {
    let client = Client::discover().await?;
    let track = client.track(123456789).await?;
    println!("{}", track.title.unwrap_or_default());
    Ok(())
}
```

`Client::new(client_id)` takes a value you already hold. `Client::discover()` reads
soundcloud.com, scans the player assets and uses the current id. Neither writes one to disk.

## Client id lifetime

Resolve once, reuse, refresh on `401`:

```rust,no_run
use soundcloud_v2::Client;

#[tokio::main]
async fn main() -> Result<(), soundcloud_v2::Error> {
    let client = Client::discover().await?;
    // Holds one id for the whole session. Every request retries once after a
    // 401 when discovery produced a different id.
    for id in [123456789_u64, 987654321] {
        let track = client.track(id).await?;
        println!("{}", track.title.unwrap_or_default());
    }
    Ok(())
}
```

- The id is cached in memory for the lifetime of the `Client`.
- A `401` triggers one re-resolve and one retry, shared across concurrent requests.
- If discovery returns the same id, the `401` is an auth gate and is returned as
  `Error::Unauthorized` without a second request.
- `refresh_client_id()` forces a refresh; `client_id()` returns the held value.
- Default request timeout is 30 seconds; override it with `with_http_client`.

## Pagination

`next_href` is absolute and, on search routes, does not carry a `client_id`. Use
`next_page`, which adds it:

```rust,no_run
use soundcloud_v2::Client;

async fn pages(client: &Client) -> Result<(), soundcloud_v2::Error> {
    let mut page = client.search_tracks("ambient").await?;
    while let Some(next) = client.next_page(&page).await? {
        page = next;
    }
    Ok(())
}
```

## Coverage

- Resolve the current client id from the web player.
- One track, several tracks by id, related tracks, track albums and track playlists.
- Search for tracks, users, playlists and albums.
- A user, a user's tracks, likes, playlists, albums, top tracks, related artists, followers and
  followings.
- A playlist or album.
- Resolve a public URL.
- Resolve a transcoding to a stream URL.
- Charts.
- Comments on a track.

The client sends `client_id` on every request. It has no user token and no write routes.

## Documentation

Protocol notes, examples, the client guide and the endpoint index are in the repository:
<https://github.com/KlugesBuch1/soundcloud-protocol>

## License

MIT.
