//! JSON data-transfer objects shared by `feedd` (responses) and `feedctl`
//! (parsing + text rendering).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedDto {
    pub id: i64,
    pub url: String,
    pub title: Option<String>,
    pub last_fetched_at: Option<String>,
    pub last_error: Option<String>,
    pub entry_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryDto {
    pub id: i64,
    pub feed_id: i64,
    pub guid: String,
    pub title: String,
    pub link: Option<String>,
    pub summary: Option<String>,
    pub published_at: Option<String>,
    pub fetched_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntriesResponse {
    pub total: i64,
    pub items: Vec<EntryDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshResult {
    /// Present for `/api/refresh` (all feeds); omitted for single-feed refresh.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feed_id: Option<i64>,
    pub status: String,
    pub new_entries: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorBody {
    pub error: String,
}
