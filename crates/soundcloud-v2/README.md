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
    let client = Client::discover().await?;
    let track = client.track(123456789).await?;
    println!("{}", track.title.unwrap_or_default());
    Ok(())
}
```

`Client::new(client_id)` takes a value you already hold. `Client::discover()` reads
soundcloud.com, scans the player assets and uses the current id. Neither writes one to disk.

## Coverage

- Resolve the current client id from the web player.
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
