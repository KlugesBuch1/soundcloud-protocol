# soundcloud-protocol

A catalogue of the HTTP API used by the SoundCloud web player. It covers `api-v2.soundcloud.com`, from tracks and streams to search, charts, comments and likes. It is unofficial and does not cover the whole API surface.

## What this does not cover

The source of truth is [endpoints.yaml](endpoints.yaml). A route that is not in that file is not part of the catalogue.

The catalogue does not cover uploads, account settings, messaging, payments or private resources. It does not execute likes or comments with a user token. It does not perform the DRM licence exchange or serve CDN media.

## Areas

| Area | File |
| --- | --- |
| Conventions | [docs/conventions.md](docs/conventions.md) |
| Authentication | [docs/auth.md](docs/auth.md) |
| Tracks and comments | [docs/tracks.md](docs/tracks.md) |
| Streams | [docs/streams.md](docs/streams.md) |
| Search | [docs/search.md](docs/search.md) |
| Users | [docs/users.md](docs/users.md) |
| Playlists | [docs/playlists.md](docs/playlists.md) |
| Charts | [docs/charts.md](docs/charts.md) |
| Likes | [docs/likes.md](docs/likes.md) |
| Errors | [docs/errors.md](docs/errors.md) |
| Endpoint index | [docs/endpoint-index.md](docs/endpoint-index.md) |

## Probe

`tools/probe.py` reads `endpoints.yaml` and sends every read route. It needs PyYAML and a client id in the `SC_CLIENT_ID` environment variable. It never resolves the id itself and never writes it to disk.

```powershell
python -m pip install pyyaml
$env:SC_CLIENT_ID = Read-Host
python tools/probe.py
```

The probe prints one line per route with the expected and actual status. It exits 1 when a status differs.

`--update` rewrites only the `status` value of a mismatched entry. The rest of the file is left as it was.

`tools/from_har.py` extracts `api-v2` routes from a HAR file, writes trimmed bodies to `examples/` and prints YAML blocks.

## License

MIT. See [LICENSE-MIT](LICENSE-MIT).
