# Endpoint index

Status is the measured response. The write routes and `/me` show 401 because no user token was sent.

## Tracks

[tracks.md](tracks.md)

| Method | Path | Auth | Status |
| --- | --- | --- | --- |
| GET | `/tracks/{track_id}` | client_id | 200 |
| GET | `/tracks` | client_id | 200 |
| GET | `/tracks/{track_id}/comments` | client_id | 200 |
| POST | `/tracks/{track_id}/comments` | user-token | 401 |
| GET | `/tracks/{track_id}/related` | client_id | 200 |
| GET | `/tracks/{track_id}/albums` | client_id | 200 |
| GET | `/tracks/{track_id}/playlists_without_albums` | client_id | 200 |
| GET | `/resolve` | client_id | 200 |

## Streams

[streams.md](streams.md)

| Method | Path | Auth | Status |
| --- | --- | --- | --- |
| GET | `/media/{media_path}` | client_id | 200 |

## Users

[users.md](users.md)

| Method | Path | Auth | Status |
| --- | --- | --- | --- |
| GET | `/users/{user_id}` | client_id | 200 |
| GET | `/users/{user_id}/tracks` | client_id | 200 |
| GET | `/users/{user_id}/likes` | client_id | 200 |
| GET | `/users/{user_id}/playlists` | client_id | 200 |
| GET | `/users/{user_id}/albums` | client_id | 200 |
| GET | `/users/{user_id}/toptracks` | client_id | 200 |
| GET | `/users/{user_id}/relatedartists` | client_id | 200 |
| GET | `/users/{user_id}/followers` | client_id | 200 |
| GET | `/users/{user_id}/followings` | client_id | 200 |

## Playlists

[playlists.md](playlists.md)

| Method | Path | Auth | Status |
| --- | --- | --- | --- |
| GET | `/playlists/{playlist_id}` | client_id | 200 |

## Search

[search.md](search.md)

| Method | Path | Auth | Status |
| --- | --- | --- | --- |
| GET | `/search/tracks` | client_id | 200 |
| GET | `/search/users` | client_id | 200 |
| GET | `/search/playlists` | client_id | 200 |
| GET | `/search/albums` | client_id | 200 |

## Charts

[charts.md](charts.md)

| Method | Path | Auth | Status |
| --- | --- | --- | --- |
| GET | `/charts` | client_id | 200 |

## Auth

[auth.md](auth.md)

| Method | Path | Auth | Status |
| --- | --- | --- | --- |
| GET | `/me` | user-token | 401 |

## Likes

[likes.md](likes.md)

| Method | Path | Auth | Status |
| --- | --- | --- | --- |
| PUT | `/users/{user_id}/track_likes/{track_id}` | user-token | 401 |
| DELETE | `/users/{user_id}/track_likes/{track_id}` | user-token | 401 |
