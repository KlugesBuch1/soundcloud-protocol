#![deny(missing_docs)]
//! Read-only client for the api-v2 routes documented in this repository.

use std::sync::RwLock;
use std::time::Duration;

use reqwest::Response;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use tokio::sync::Mutex as AsyncMutex;

const DEFAULT_BASE_URL: &str = "https://api-v2.soundcloud.com";
const PLAYER_URL: &str = "https://soundcloud.com/";
const ASSET_PREFIX: &str = "https://a-v2.sndcdn.com/assets/";
const ID_MARKER: &str = "client_id:\"";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const BROWSER_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

/// Client for the measured read routes.
///
/// The client id is resolved once and kept in memory for the lifetime of the
/// value. A request that comes back `401` triggers one re-resolve and one retry
/// with the new id. Nothing is written to disk.
pub struct Client {
    http: reqwest::Client,
    client_id: RwLock<String>,
    refresh: AsyncMutex<()>,
    base_url: String,
    player_url: String,
    asset_prefix: String,
}

impl Client {
    /// Creates a client with a client id.
    pub fn new(client_id: impl Into<String>) -> Self {
        let http = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self::with_http(http, client_id.into())
    }

    fn with_http(http: reqwest::Client, client_id: String) -> Self {
        Self {
            http,
            client_id: RwLock::new(client_id),
            refresh: AsyncMutex::new(()),
            base_url: DEFAULT_BASE_URL.to_string(),
            player_url: PLAYER_URL.to_string(),
            asset_prefix: ASSET_PREFIX.to_string(),
        }
    }

    /// Replaces the HTTP client, for a custom timeout, proxy or TLS setup.
    pub fn with_http_client(mut self, http: reqwest::Client) -> Self {
        self.http = http;
        self
    }

    /// Replaces the API base URL.
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into().trim_end_matches('/').to_string();
        self
    }

    /// Replaces the player page and asset prefix used to resolve a client id.
    ///
    /// The defaults point at `soundcloud.com` and `a-v2.sndcdn.com`. Override
    /// these to run against a mirror or a test server.
    pub fn with_discovery_urls(
        mut self,
        player_url: impl Into<String>,
        asset_prefix: impl Into<String>,
    ) -> Self {
        self.player_url = player_url.into();
        self.asset_prefix = asset_prefix.into();
        self
    }

    /// Returns the client id currently held in memory.
    pub fn client_id(&self) -> String {
        self.client_id
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    /// Resolves a fresh client id from the player assets and replaces the held one.
    ///
    /// Refreshes are serialised, so two calls do not run discovery at the same
    /// time. A discovery failure leaves the held id untouched.
    pub async fn refresh_client_id(&self) -> Result<String, Error> {
        let _guard = self.refresh.lock().await;
        let id = discover_client_id(&self.http, &self.player_url, &self.asset_prefix).await?;
        self.store_client_id(&id);
        Ok(id)
    }

    /// Creates a client by resolving the current client id from the web player.
    ///
    /// Reads the landing page, fetches the player assets it references and uses
    /// the first `client_id` literal it finds. The value is kept in memory.
    pub async fn discover() -> Result<Self, Error> {
        let http = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()?;
        let id = discover_client_id(&http, PLAYER_URL, ASSET_PREFIX).await?;
        Ok(Self::with_http(http, id))
    }

    /// Returns one track.
    pub async fn track(&self, track_id: u64) -> Result<Track, Error> {
        self.get(&format!("/tracks/{track_id}"), &[]).await
    }

    /// Returns tracks by id.
    pub async fn tracks(&self, track_ids: &[u64]) -> Result<Vec<Track>, Error> {
        let ids = track_ids
            .iter()
            .map(u64::to_string)
            .collect::<Vec<_>>()
            .join(",");
        self.get("/tracks", &[("ids", ids)]).await
    }

    /// Returns tracks related to a track.
    pub async fn related_tracks(&self, track_id: u64) -> Result<Page<Track>, Error> {
        self.get(&format!("/tracks/{track_id}/related"), &[]).await
    }

    /// Returns albums a track appears on.
    pub async fn track_albums(&self, track_id: u64) -> Result<Page<Playlist>, Error> {
        self.get(&format!("/tracks/{track_id}/albums"), &[]).await
    }

    /// Returns playlists a track appears on, excluding albums.
    pub async fn track_playlists_without_albums(
        &self,
        track_id: u64,
    ) -> Result<Page<Playlist>, Error> {
        self.get(&format!("/tracks/{track_id}/playlists_without_albums"), &[])
            .await
    }

    /// Searches tracks.
    pub async fn search_tracks(&self, query: &str) -> Result<SearchPage<Track>, Error> {
        self.get("/search/tracks", &[("q", query.to_string())])
            .await
    }

    /// Searches users.
    pub async fn search_users(&self, query: &str) -> Result<SearchPage<User>, Error> {
        self.get("/search/users", &[("q", query.to_string())]).await
    }

    /// Searches playlists.
    pub async fn search_playlists(&self, query: &str) -> Result<SearchPage<Playlist>, Error> {
        self.get("/search/playlists", &[("q", query.to_string())])
            .await
    }

    /// Searches albums.
    pub async fn search_albums(&self, query: &str) -> Result<SearchPage<Playlist>, Error> {
        self.get("/search/albums", &[("q", query.to_string())])
            .await
    }

    /// Returns one user.
    pub async fn user(&self, user_id: u64) -> Result<User, Error> {
        self.get(&format!("/users/{user_id}"), &[]).await
    }

    /// Returns a user's tracks.
    pub async fn user_tracks(&self, user_id: u64) -> Result<Page<Track>, Error> {
        self.get(&format!("/users/{user_id}/tracks"), &[]).await
    }

    /// Returns a user's liked tracks.
    pub async fn user_likes(&self, user_id: u64) -> Result<Page<Track>, Error> {
        self.get(&format!("/users/{user_id}/likes"), &[]).await
    }

    /// Returns a user's playlists.
    pub async fn user_playlists(&self, user_id: u64) -> Result<Page<Playlist>, Error> {
        self.get(&format!("/users/{user_id}/playlists"), &[]).await
    }

    /// Returns a user's albums.
    pub async fn user_albums(&self, user_id: u64) -> Result<Page<Playlist>, Error> {
        self.get(&format!("/users/{user_id}/albums"), &[]).await
    }

    /// Returns a user's top tracks.
    pub async fn user_toptracks(&self, user_id: u64) -> Result<Page<Track>, Error> {
        self.get(&format!("/users/{user_id}/toptracks"), &[]).await
    }

    /// Returns artists related to a user.
    pub async fn user_related_artists(&self, user_id: u64) -> Result<Page<User>, Error> {
        self.get(&format!("/users/{user_id}/relatedartists"), &[])
            .await
    }

    /// Returns a user's followers.
    pub async fn user_followers(&self, user_id: u64) -> Result<Page<User>, Error> {
        self.get(&format!("/users/{user_id}/followers"), &[]).await
    }

    /// Returns the users a user follows.
    pub async fn user_followings(&self, user_id: u64) -> Result<Page<User>, Error> {
        self.get(&format!("/users/{user_id}/followings"), &[]).await
    }

    /// Returns one playlist or album.
    pub async fn playlist(&self, playlist_id: u64) -> Result<Playlist, Error> {
        self.get(&format!("/playlists/{playlist_id}"), &[]).await
    }

    /// Resolves a public track or playlist URL.
    pub async fn resolve(&self, url: &str) -> Result<Resolved, Error> {
        self.get("/resolve", &[("url", url.to_string())]).await
    }

    /// Resolves a transcoding to a stream URL.
    pub async fn resolve_transcoding(&self, transcoding: &Transcoding) -> Result<StreamUrl, Error> {
        let response = self
            .send(|id| self.http.get(with_client_id(&transcoding.url, id)))
            .await?;
        self.decode(response).await
    }

    /// Fetches the next page of a list response.
    ///
    /// `next_href` is absolute and does not carry a client id on the search
    /// routes, so the id is added here. Returns `None` at the end of the list.
    pub async fn next_page<T>(&self, page: &Page<T>) -> Result<Option<Page<T>>, Error>
    where
        T: DeserializeOwned,
    {
        match page.next_href.as_deref() {
            Some(href) => self.get_url(href).await.map(Some),
            None => Ok(None),
        }
    }

    async fn send(
        &self,
        build: impl Fn(&str) -> reqwest::RequestBuilder,
    ) -> Result<Response, Error> {
        let id = self.client_id();
        let response = build(&id).send().await?;
        if response.status() != reqwest::StatusCode::UNAUTHORIZED {
            return Ok(response);
        }
        match self.fresh_client_id(&id).await? {
            Some(fresh) => Ok(build(&fresh).send().await?),
            None => Ok(response),
        }
    }

    /// Re-resolves the id after a `401`. Returns `None` when discovery produced
    /// the same id, which means the `401` came from a missing user token.
    async fn fresh_client_id(&self, stale: &str) -> Result<Option<String>, Error> {
        let _guard = self.refresh.lock().await;
        let current = self.client_id();
        if current != stale {
            return Ok(Some(current));
        }
        let discovered =
            discover_client_id(&self.http, &self.player_url, &self.asset_prefix).await?;
        if discovered == stale {
            return Ok(None);
        }
        self.store_client_id(&discovered);
        Ok(Some(discovered))
    }

    fn store_client_id(&self, id: &str) {
        let mut held = self
            .client_id
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *held = id.to_string();
    }

    async fn get<T>(&self, path: &str, query: &[(&str, String)]) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        let response = self
            .send(|id| {
                self.http
                    .get(format!("{}{}", self.base_url, path))
                    .query(&[("client_id", id)])
                    .query(query)
            })
            .await?;
        self.decode(response).await
    }

    async fn get_url<T>(&self, url: &str) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        let response = self
            .send(|id| self.http.get(with_client_id(url, id)))
            .await?;
        self.decode(response).await
    }

    /// Returns a chart page.
    pub async fn charts(&self, kind: &str, genre: &str) -> Result<ChartPage, Error> {
        self.get(
            "/charts",
            &[("kind", kind.to_string()), ("genre", genre.to_string())],
        )
        .await
    }

    /// Returns comments on a track.
    pub async fn comments(
        &self,
        track_id: u64,
        limit: Option<u32>,
    ) -> Result<Page<Comment>, Error> {
        let mut query = vec![("threaded", "1".to_string())];
        if let Some(limit) = limit {
            query.push(("limit", limit.to_string()));
        }
        self.get(&format!("/tracks/{track_id}/comments"), &query)
            .await
    }

    async fn decode<T>(&self, response: Response) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        let status = response.status();
        if status.is_success() {
            return Ok(response.json::<T>().await?);
        }
        let body = response.text().await.unwrap_or_default();
        match status.as_u16() {
            401 => Err(Error::Unauthorized),
            404 => Err(Error::NotFound),
            code => Err(Error::Status { status: code, body }),
        }
    }
}

/// One error type for the client.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The service returned 401.
    #[error("unauthorized")]
    Unauthorized,
    /// The service returned 404.
    #[error("not found")]
    NotFound,
    /// The service returned another non-success status.
    #[error("HTTP {status}: {body}")]
    Status {
        /// HTTP status code.
        status: u16,
        /// Response body.
        body: String,
    },
    /// The client id could not be resolved from the web player.
    #[error("client id discovery failed: {0}")]
    Discovery(String),
    /// The request failed before a response was decoded.
    #[error(transparent)]
    Request(#[from] reqwest::Error),
}

/// A track returned by the measured routes.
#[derive(Debug, Clone, Deserialize)]
pub struct Track {
    /// Numeric id.
    pub id: u64,
    /// Object kind.
    pub kind: Option<String>,
    /// URN.
    pub urn: Option<String>,
    /// API URI.
    pub uri: Option<String>,
    /// Title.
    pub title: Option<String>,
    /// Public URL.
    pub permalink_url: Option<String>,
    /// Artwork URL.
    pub artwork_url: Option<String>,
    /// Creation timestamp.
    pub created_at: Option<String>,
    /// Display timestamp.
    pub display_date: Option<String>,
    /// Last modification timestamp.
    pub last_modified: Option<String>,
    /// Release timestamp.
    pub release_date: Option<String>,
    /// Playback duration in milliseconds.
    pub duration: Option<u64>,
    /// Full duration in milliseconds.
    pub full_duration: Option<u64>,
    /// Genre.
    pub genre: Option<String>,
    /// License name.
    pub license: Option<String>,
    /// Stream flag.
    pub streamable: Option<bool>,
    /// Playback policy.
    pub policy: Option<String>,
    /// Monetization model.
    pub monetization_model: Option<String>,
    /// Download flag.
    pub downloadable: Option<bool>,
    /// Download count.
    pub download_count: Option<u64>,
    /// Comment flag.
    pub commentable: Option<bool>,
    /// Comment count.
    pub comment_count: Option<u64>,
    /// Like count.
    pub likes_count: Option<u64>,
    /// Play count.
    pub playback_count: Option<u64>,
    /// Repost count.
    pub reposts_count: Option<u64>,
    /// Owner id.
    pub user_id: Option<u64>,
    /// Owner.
    pub user: Option<User>,
    /// Stream media.
    pub media: Option<Media>,
}

/// Stream media for a track.
#[derive(Debug, Clone, Deserialize)]
pub struct Media {
    /// Transcoding entries.
    #[serde(default)]
    pub transcodings: Vec<Transcoding>,
}

/// One transcoding entry.
#[derive(Debug, Clone, Deserialize)]
pub struct Transcoding {
    /// Media URL.
    pub url: String,
    /// Encoding preset.
    pub preset: Option<String>,
    /// Rendition duration in milliseconds.
    pub duration: Option<u64>,
    /// Preview flag.
    pub snipped: Option<bool>,
    /// Quality label.
    pub quality: Option<String>,
    /// Format details.
    pub format: Option<TranscodingFormat>,
}

/// Format details for a transcoding.
#[derive(Debug, Clone, Deserialize)]
pub struct TranscodingFormat {
    /// Protocol name.
    pub protocol: Option<String>,
    /// MIME type.
    pub mime_type: Option<String>,
}

/// A resolved stream URL.
#[derive(Debug, Clone, Deserialize)]
pub struct StreamUrl {
    /// Signed media URL.
    pub url: String,
}

/// A user returned by the measured routes.
#[derive(Debug, Clone, Deserialize)]
pub struct User {
    /// Numeric id.
    pub id: u64,
    /// Object kind.
    pub kind: Option<String>,
    /// URN.
    pub urn: Option<String>,
    /// API URI.
    pub uri: Option<String>,
    /// Username.
    pub username: Option<String>,
    /// Profile slug.
    pub permalink: Option<String>,
    /// Public profile URL.
    pub permalink_url: Option<String>,
    /// Full name.
    pub full_name: Option<String>,
    /// Profile text.
    pub description: Option<String>,
    /// Avatar URL.
    pub avatar_url: Option<String>,
    /// City.
    pub city: Option<String>,
    /// Country code.
    pub country_code: Option<String>,
    /// Creation timestamp.
    pub created_at: Option<String>,
    /// Last modification timestamp.
    pub last_modified: Option<String>,
    /// Track count.
    pub track_count: Option<u64>,
    /// Playlist count.
    pub playlist_count: Option<u64>,
    /// Follower count.
    pub followers_count: Option<u64>,
    /// Following count.
    pub followings_count: Option<u64>,
    /// Like count.
    pub likes_count: Option<u64>,
    /// Repost count.
    pub reposts_count: Option<u64>,
    /// Comment count.
    pub comments_count: Option<u64>,
    /// Verification flag.
    pub verified: Option<bool>,
}

/// A playlist or album.
#[derive(Debug, Clone, Deserialize)]
pub struct Playlist {
    /// Numeric id.
    pub id: u64,
    /// Object kind.
    pub kind: Option<String>,
    /// Album flag.
    pub is_album: Option<bool>,
    /// Set type.
    pub set_type: Option<String>,
    /// Track count.
    pub track_count: Option<u64>,
    /// Title.
    pub title: Option<String>,
    /// Public URL.
    pub permalink_url: Option<String>,
    /// Artwork URL.
    pub artwork_url: Option<String>,
    /// Creation timestamp.
    pub created_at: Option<String>,
    /// Release timestamp.
    pub release_date: Option<String>,
    /// Publish timestamp.
    pub published_at: Option<String>,
    /// Display timestamp.
    pub display_date: Option<String>,
    /// Duration in milliseconds.
    pub duration: Option<u64>,
    /// Genre.
    pub genre: Option<String>,
    /// Label.
    pub label_name: Option<String>,
    /// Owner id.
    pub user_id: Option<u64>,
    /// Owner.
    pub user: Option<User>,
    /// Tracks.
    #[serde(default)]
    pub tracks: Vec<Track>,
}

/// A page of objects.
#[derive(Debug, Clone, Deserialize)]
pub struct Page<T> {
    /// Page objects.
    pub collection: Vec<T>,
    /// Total results, when supplied.
    pub total_results: Option<u64>,
    /// Next page URL.
    pub next_href: Option<String>,
    /// Query cursor.
    pub query_urn: Option<String>,
}

/// A search page.
pub type SearchPage<T> = Page<T>;

/// A comment.
#[derive(Debug, Clone, Deserialize)]
pub struct Comment {
    /// Numeric id.
    pub id: u64,
    /// Object kind.
    pub kind: Option<String>,
    /// Comment text.
    pub body: String,
    /// Creation timestamp.
    pub created_at: Option<String>,
    /// Position in milliseconds.
    pub timestamp: Option<u64>,
    /// Track id.
    pub track_id: Option<u64>,
    /// Author id.
    pub user_id: Option<u64>,
    /// Author.
    pub user: Option<User>,
}

/// A chart page.
#[derive(Debug, Clone, Deserialize)]
pub struct ChartPage {
    /// Requested genre.
    pub genre: Option<String>,
    /// Requested kind.
    pub kind: Option<String>,
    /// Last update timestamp.
    pub last_updated: Option<String>,
    /// Chart entries.
    #[serde(default)]
    pub collection: Vec<ChartEntry>,
    /// Query cursor.
    pub query_urn: Option<String>,
    /// Next page URL.
    pub next_href: Option<String>,
}

/// One chart entry.
#[derive(Debug, Clone, Deserialize)]
pub struct ChartEntry {
    /// Chart score.
    pub score: f64,
    /// Charted track.
    pub track: Track,
}

/// A resolved track or playlist.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Resolved {
    /// A track.
    Track(Track),
    /// A playlist.
    Playlist(Playlist),
}

async fn discover_client_id(
    http: &reqwest::Client,
    player_url: &str,
    asset_prefix: &str,
) -> Result<String, Error> {
    let response = http
        .get(player_url)
        .header(reqwest::header::USER_AGENT, BROWSER_USER_AGENT)
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(Error::Discovery(format!(
            "the player page returned HTTP {}",
            response.status().as_u16()
        )));
    }
    let html = response.text().await?;
    let urls = script_urls(&html, asset_prefix);
    if urls.is_empty() {
        return Err(Error::Discovery(
            "the player page referenced no assets".to_string(),
        ));
    }
    for url in urls {
        let response = http
            .get(&url)
            .header(reqwest::header::USER_AGENT, BROWSER_USER_AGENT)
            .send()
            .await?;
        if !response.status().is_success() {
            continue;
        }
        let script = response.text().await?;
        if let Some(id) = find_client_id(&script) {
            return Ok(id.to_string());
        }
    }
    Err(Error::Discovery(
        "no client id in the player assets".to_string(),
    ))
}

fn with_client_id(url: &str, client_id: &str) -> String {
    if url.contains("client_id=") {
        return url.to_string();
    }
    let separator = if url.contains('?') { '&' } else { '?' };
    format!("{url}{separator}client_id={client_id}")
}

fn script_urls(html: &str, asset_prefix: &str) -> Vec<String> {
    let mut urls: Vec<String> = Vec::new();
    let mut rest = html;
    while let Some(start) = rest.find(asset_prefix) {
        let tail = &rest[start..];
        let Some(end) = tail.find('"') else {
            break;
        };
        let url = &tail[..end];
        if url.ends_with(".js") && !urls.iter().any(|seen| seen.as_str() == url) {
            urls.push(url.to_string());
        }
        rest = &tail[end..];
    }
    urls
}

fn find_client_id(script: &str) -> Option<&str> {
    let mut rest = script;
    while let Some(start) = rest.find(ID_MARKER) {
        let tail = &rest[start + ID_MARKER.len()..];
        if tail.len() > 32 && tail.is_char_boundary(32) && tail.as_bytes()[32] == b'"' {
            let id = &tail[..32];
            if id.bytes().all(|byte| byte.is_ascii_alphanumeric()) {
                return Some(id);
            }
        }
        rest = tail;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn load<T: DeserializeOwned>(name: &str) -> T {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples")
            .join(name);
        let text = fs::read_to_string(path).expect("read example");
        serde_json::from_str(&text).expect("parse example")
    }

    const FAKE_ID: &str = "0123456789abcdefghijklmnopqrstuv";

    #[test]
    fn extracts_script_urls() {
        let html = r#"<script src="https://a-v2.sndcdn.com/assets/59-ac0a49ce.js"></script>
<script src="https://example.com/app.js"></script>
<script src="https://a-v2.sndcdn.com/assets/55-a70a06aa.js"></script>
<script src="https://a-v2.sndcdn.com/assets/59-ac0a49ce.js"></script>"#;
        assert_eq!(
            script_urls(html, ASSET_PREFIX),
            vec![
                "https://a-v2.sndcdn.com/assets/59-ac0a49ce.js",
                "https://a-v2.sndcdn.com/assets/55-a70a06aa.js",
            ]
        );
    }

    #[test]
    fn extracts_client_id() {
        let script = format!(r#"e.client_id:"{FAKE_ID}",t"#);
        assert_eq!(find_client_id(&script), Some(FAKE_ID));
    }

    #[test]
    fn skips_malformed_client_ids() {
        assert_eq!(find_client_id(r#"client_id:"short""#), None);
        assert_eq!(
            find_client_id(r#"client_id:"0123456789abcdefghijklmnopqrstu-""#),
            None
        );
        assert_eq!(find_client_id("no marker"), None);
        assert_eq!(find_client_id(&format!(r#"client_id:"{FAKE_ID}x""#)), None);
        assert_eq!(
            find_client_id(&format!(r#"client_id:"short",client_id:"{FAKE_ID}""#)),
            Some(FAKE_ID)
        );
    }

    #[test]
    fn parses_track() {
        let track: Track = load("track.json");
        assert_eq!(track.id, 123456789);
        assert_eq!(track.media.expect("media").transcodings.len(), 2);
    }

    #[test]
    fn parses_snipped_track() {
        let track: Track = load("track-snipped.json");
        assert_eq!(track.policy.as_deref(), Some("SNIP"));
        assert_eq!(track.duration, Some(30000));
    }

    #[test]
    fn parses_blocked_track() {
        let track: Track = load("track-blocked.json");
        assert_eq!(track.policy.as_deref(), Some("BLOCK"));
        assert!(track.media.expect("media").transcodings.is_empty());
    }

    #[test]
    fn parses_encrypted_track() {
        let track: Track = load("track-encrypted.json");
        let media = track.media.expect("media");
        assert_eq!(media.transcodings.len(), 2);
    }

    #[test]
    fn parses_user() {
        let user: User = load("user.json");
        assert_eq!(user.username.as_deref(), Some("Example Artist"));
    }

    #[test]
    fn parses_playlist() {
        let playlist: Playlist = load("playlist.json");
        assert_eq!(playlist.is_album, Some(false));
        assert_eq!(playlist.tracks.len(), 2);
    }

    #[test]
    fn parses_album() {
        let playlist: Playlist = load("album.json");
        assert_eq!(playlist.is_album, Some(true));
    }

    #[test]
    fn parses_search_tracks() {
        let page: SearchPage<Track> = load("search-tracks.json");
        assert_eq!(page.total_results, Some(1));
        assert_eq!(page.collection.len(), 1);
    }

    #[test]
    fn parses_search_users() {
        let page: SearchPage<User> = load("search-users.json");
        assert_eq!(page.collection.len(), 1);
    }

    #[test]
    fn parses_search_playlists() {
        let page: SearchPage<Playlist> = load("search-playlists.json");
        assert_eq!(page.collection.len(), 2);
    }

    #[test]
    fn parses_track_list() {
        let tracks: Vec<Track> = load("tracks.json");
        assert_eq!(tracks.len(), 2);
    }

    #[test]
    fn parses_user_tracks() {
        let page: Page<Track> = load("user-tracks.json");
        assert_eq!(page.collection.len(), 2);
    }

    #[test]
    fn parses_comments() {
        let page: Page<Comment> = load("comments.json");
        assert_eq!(page.collection.len(), 2);
    }

    #[test]
    fn parses_charts() {
        let page: ChartPage = load("charts.json");
        assert_eq!(page.collection.len(), 2);
    }

    #[test]
    fn parses_progressive_url() {
        let stream: StreamUrl = load("media-progressive.json");
        assert!(stream.url.ends_with(".mp3"));
    }

    #[test]
    fn parses_hls_url() {
        let stream: StreamUrl = load("media-hls.json");
        assert!(stream.url.ends_with(".m3u8"));
    }

    #[test]
    fn parses_resolved_track() {
        let resolved: Resolved = load("track.json");
        assert!(matches!(resolved, Resolved::Track(_)));
    }

    #[test]
    fn parses_resolved_playlist() {
        let resolved: Resolved = load("playlist.json");
        assert!(matches!(resolved, Resolved::Playlist(_)));
    }

    const FRESH_ID: &str = "zyxwvutsrqponmlkjihgfedcba987654";

    #[test]
    fn appends_client_id_when_missing() {
        assert_eq!(
            with_client_id("https://api-v2.soundcloud.com/tracks/1", "abc"),
            "https://api-v2.soundcloud.com/tracks/1?client_id=abc"
        );
        assert_eq!(
            with_client_id("https://api-v2.soundcloud.com/search/tracks?q=a", "abc"),
            "https://api-v2.soundcloud.com/search/tracks?q=a&client_id=abc"
        );
        assert_eq!(
            with_client_id("https://api-v2.soundcloud.com/tracks?client_id=old", "abc"),
            "https://api-v2.soundcloud.com/tracks?client_id=old"
        );
    }

    #[test]
    fn exposes_the_held_client_id() {
        let client = Client::new(FAKE_ID);
        assert_eq!(client.client_id(), FAKE_ID);
    }

    #[tokio::test]
    async fn next_page_stops_at_the_end() {
        let client = Client::new(FAKE_ID);
        let page: Page<Track> = Page {
            collection: Vec::new(),
            total_results: None,
            next_href: None,
            query_urn: None,
        };
        assert!(client.next_page(&page).await.expect("page").is_none());
    }

    async fn spawn_server<F>(handler: F) -> String
    where
        F: Fn(&str) -> (u16, String) + Clone + Send + 'static,
    {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind test server");
        let address = listener.local_addr().expect("test server address");
        tokio::spawn(async move {
            loop {
                let Ok((mut stream, _)) = listener.accept().await else {
                    break;
                };
                let handler = handler.clone();
                tokio::spawn(async move {
                    let mut buf = vec![0_u8; 8192];
                    let mut filled = 0;
                    loop {
                        let read = stream.read(&mut buf[filled..]).await.unwrap_or(0);
                        if read == 0 {
                            break;
                        }
                        filled += read;
                        if buf[..filled].windows(4).any(|window| window == b"\r\n\r\n") {
                            break;
                        }
                    }
                    let request = String::from_utf8_lossy(&buf[..filled]).to_string();
                    let (status, body) = handler(&request);
                    let reason = match status {
                        200 => "OK",
                        401 => "Unauthorized",
                        _ => "Not Found",
                    };
                    let response = format!(
                        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                        body.len()
                    );
                    let _ = stream.write_all(response.as_bytes()).await;
                    let _ = stream.shutdown().await;
                });
            }
        });
        format!("http://{address}")
    }

    fn request_path(request: &str) -> &str {
        request
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(1))
            .unwrap_or("/")
    }

    fn request_host(request: &str) -> &str {
        request
            .lines()
            .find(|line| line.len() > 5 && line[..5].eq_ignore_ascii_case("host:"))
            .map(|line| line[5..].trim())
            .unwrap_or("127.0.0.1")
    }

    #[tokio::test]
    async fn refreshes_the_client_id_once_and_reuses_it() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let discoveries = Arc::new(AtomicUsize::new(0));
        let counter = discoveries.clone();
        let base = spawn_server(move |request| {
            let path = request_path(request).to_string();
            let host = request_host(request).to_string();
            if path == "/player" {
                counter.fetch_add(1, Ordering::SeqCst);
                return (
                    200,
                    format!(r#"<script src="http://{host}/assets/app.js"></script>"#),
                );
            }
            if path == "/assets/app.js" {
                return (200, format!(r#"e.client_id:"{FRESH_ID}""#));
            }
            if path.starts_with("/tracks/1") {
                if path.contains("client_id=stale") {
                    return (401, String::new());
                }
                return (200, r#"{"id":1,"kind":"track","title":"ok"}"#.to_string());
            }
            (404, String::new())
        })
        .await;

        let client = Client::new("stale")
            .with_base_url(&base)
            .with_discovery_urls(format!("{base}/player"), format!("{base}/assets/"));

        let track = client.track(1).await.expect("first track");
        assert_eq!(track.id, 1);
        assert_eq!(client.client_id(), FRESH_ID);

        let track = client.track(1).await.expect("second track");
        assert_eq!(track.id, 1);
        assert_eq!(
            discoveries.load(Ordering::SeqCst),
            1,
            "the client id is resolved once per session, not per request"
        );
    }

    #[tokio::test]
    async fn keeps_unauthorized_when_discovery_finds_the_same_id() {
        let base = spawn_server(|request| {
            let path = request_path(request).to_string();
            let host = request_host(request).to_string();
            if path == "/player" {
                return (
                    200,
                    format!(r#"<script src="http://{host}/assets/app.js"></script>"#),
                );
            }
            if path == "/assets/app.js" {
                return (200, format!(r#"e.client_id:"{FAKE_ID}""#));
            }
            (401, String::new())
        })
        .await;

        let client = Client::new(FAKE_ID)
            .with_base_url(&base)
            .with_discovery_urls(format!("{base}/player"), format!("{base}/assets/"));

        assert!(matches!(client.track(1).await, Err(Error::Unauthorized)));
    }

    #[tokio::test]
    async fn next_page_adds_the_client_id_to_next_href() {
        let base = spawn_server(|request| {
            let path = request_path(request).to_string();
            if path.contains("/search/tracks")
                && path.contains("offset=2")
                && path.contains("client_id=")
            {
                return (
                    200,
                    r#"{"collection":[{"id":3}],"next_href":null}"#.to_string(),
                );
            }
            (401, String::new())
        })
        .await;

        let client = Client::new(FAKE_ID).with_base_url(&base);
        let page: Page<Track> = Page {
            collection: Vec::new(),
            total_results: None,
            next_href: Some(format!("{base}/search/tracks?q=x&offset=2")),
            query_urn: None,
        };

        let next = client
            .next_page(&page)
            .await
            .expect("next page")
            .expect("a page");
        assert_eq!(next.collection[0].id, 3);
        assert!(next.next_href.is_none());
    }
}
