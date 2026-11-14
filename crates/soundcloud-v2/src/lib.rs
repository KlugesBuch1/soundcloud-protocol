#![deny(missing_docs)]
//! Read-only client for the api-v2 routes documented in this repository.

use std::time::Duration;

use reqwest::Response;
use serde::de::DeserializeOwned;
use serde::Deserialize;

const DEFAULT_BASE_URL: &str = "https://api-v2.soundcloud.com";
const PLAYER_URL: &str = "https://soundcloud.com/";
const ASSET_PREFIX: &str = "https://a-v2.sndcdn.com/assets/";
const ID_MARKER: &str = "client_id:\"";
const BROWSER_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

/// Client for the measured read routes.
pub struct Client {
    http: reqwest::Client,
    client_id: String,
    base_url: String,
}

impl Client {
    /// Creates a client with a client id.
    pub fn new(client_id: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            client_id,
            base_url: DEFAULT_BASE_URL.to_string(),
        }
    }

    /// Replaces the API base URL.
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into().trim_end_matches('/').to_string();
        self
    }

    /// Creates a client by resolving the current client id from the web player.
    ///
    /// Reads the landing page, fetches the player assets it references and uses
    /// the first `client_id` literal it finds. The value is kept in memory.
    pub async fn discover() -> Result<Self, Error> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;
        let response = http
            .get(PLAYER_URL)
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
        let urls = script_urls(&html);
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
                let mut client = Client::new(id.to_string());
                client.http = http;
                return Ok(client);
            }
        }
        Err(Error::Discovery(
            "no client id in the player assets".to_string(),
        ))
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

    /// Searches tracks.
    pub async fn search_tracks(&self, query: &str) -> Result<SearchPage<Track>, Error> {
        self.get("/search/tracks", &[("q", query.to_string())])
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
            .http
            .get(&transcoding.url)
            .query(&[("client_id", self.client_id.as_str())])
            .send()
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

    async fn get<T>(&self, path: &str, query: &[(&str, String)]) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        let response = self
            .http
            .get(format!("{}{}", self.base_url, path))
            .query(&[("client_id", self.client_id.as_str())])
            .query(query)
            .send()
            .await?;
        self.decode(response).await
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
    /// Last modification timestamp.
    pub last_modified: Option<String>,
    /// Release timestamp.
    pub release_date: Option<String>,
    /// Playback duration in milliseconds.
    pub duration: Option<u64>,
    /// Full duration in milliseconds.
    pub full_duration: Option<u64>,
    /// Stream flag.
    pub streamable: Option<bool>,
    /// Playback policy.
    pub policy: Option<String>,
    /// Monetization model.
    pub monetization_model: Option<String>,
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

fn script_urls(html: &str) -> Vec<String> {
    let mut urls: Vec<String> = Vec::new();
    let mut rest = html;
    while let Some(start) = rest.find(ASSET_PREFIX) {
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
            script_urls(html),
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
}
