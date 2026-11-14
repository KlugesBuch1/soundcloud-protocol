# soundcloud-v2

Read-only Rust client for the `api-v2.soundcloud.com` routes in the catalogue: tracks, tracks by id,
search, users, playlists, resolve, transcoding resolve, charts and comments.

Unofficial. Not affiliated with SoundCloud.

## Install

```toml
[dependencies]
soundcloud-v2 = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## Use

```rust,no_run
use soundcloud_v2::Client;

#[tokio::main]
async fn main() -> Result<(), soundcloud_v2::Error> {
    let client = Client::new(std::env::var("SC_CLIENT_ID").unwrap());
    let track = client.track(123456789).await?;
    println!("{}", track.title.unwrap_or_default());
    Ok(())
}
```

## Coverage

- One track, and several tracks by id.
- Search for tracks.
- A user, and a user's tracks.
- A playlist or album.
- Resolve a public URL.
- Resolve a transcoding to a stream URL.
- Charts.
- Comments on a track.

The client sends `client_id` on every request. It has no user token and no write routes.

## Documentation

The protocol notes, examples and endpoint index are in the repository:
<https://github.com/KlugesBuch1/soundcloud-protocol>

## License

MIT.
