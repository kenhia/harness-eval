//! REST API: axum router and handlers.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;

use feedcore::date::to_rfc3339_z;
use feedcore::parse_date;

use crate::db::{self, EntryQuery, InsertOutcome};
use crate::fetch;
use crate::state::AppState;

/// Build the application router.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/feeds", post(create_feed).get(list_feeds))
        .route("/api/feeds/:id", get(get_feed).delete(delete_feed))
        .route("/api/feeds/:id/refresh", post(refresh_one))
        .route("/api/refresh", post(refresh_all))
        .route("/api/entries", get(get_entries))
        .with_state(state)
}

fn api_error(status: StatusCode, msg: impl Into<String>) -> Response {
    (status, Json(json!({ "error": msg.into() }))).into_response()
}

async fn health() -> Response {
    Json(json!({ "status": "ok" })).into_response()
}

#[derive(Deserialize)]
struct CreateFeed {
    url: String,
}

async fn create_feed(State(state): State<AppState>, body: Option<Json<CreateFeed>>) -> Response {
    let Some(Json(body)) = body else {
        return api_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "expected JSON body {\"url\": ...}",
        );
    };
    let url = body.url.trim().to_string();
    if !valid_http_url(&url) {
        return api_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "url must be a valid http(s) URL",
        );
    }
    let conn = state.db.lock().unwrap();
    match db::insert_feed(&conn, &url) {
        Ok(InsertOutcome::Created(feed)) => (StatusCode::CREATED, Json(feed)).into_response(),
        Ok(InsertOutcome::Conflict) => {
            api_error(StatusCode::CONFLICT, "feed URL already registered")
        }
        Err(e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

async fn list_feeds(State(state): State<AppState>) -> Response {
    let conn = state.db.lock().unwrap();
    match db::list_feeds(&conn) {
        Ok(feeds) => Json(feeds).into_response(),
        Err(e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

async fn get_feed(State(state): State<AppState>, Path(id): Path<i64>) -> Response {
    let conn = state.db.lock().unwrap();
    match db::get_feed(&conn, id) {
        Ok(Some(feed)) => Json(feed).into_response(),
        Ok(None) => api_error(StatusCode::NOT_FOUND, "feed not found"),
        Err(e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

async fn delete_feed(State(state): State<AppState>, Path(id): Path<i64>) -> Response {
    let conn = state.db.lock().unwrap();
    match db::delete_feed(&conn, id) {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => api_error(StatusCode::NOT_FOUND, "feed not found"),
        Err(e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

async fn refresh_one(State(state): State<AppState>, Path(id): Path<i64>) -> Response {
    let exists = {
        let conn = state.db.lock().unwrap();
        matches!(db::get_feed(&conn, id), Ok(Some(_)))
    };
    if !exists {
        return api_error(StatusCode::NOT_FOUND, "feed not found");
    }
    let result = fetch::refresh_feed(&state, id).await;
    Json(result).into_response()
}

async fn refresh_all(State(state): State<AppState>) -> Response {
    let results = fetch::refresh_all(&state).await;
    Json(results).into_response()
}

#[derive(Deserialize)]
struct EntryParams {
    feed_id: Option<i64>,
    since: Option<String>,
    until: Option<String>,
    q: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
}

async fn get_entries(State(state): State<AppState>, Query(p): Query<EntryParams>) -> Response {
    let since = match normalize_bound(p.since.as_deref()) {
        Ok(v) => v,
        Err(_) => return api_error(StatusCode::BAD_REQUEST, "invalid 'since' timestamp"),
    };
    let until = match normalize_bound(p.until.as_deref()) {
        Ok(v) => v,
        Err(_) => return api_error(StatusCode::BAD_REQUEST, "invalid 'until' timestamp"),
    };
    let limit = p.limit.unwrap_or(50).clamp(0, 500);
    let offset = p.offset.unwrap_or(0).max(0);
    let q = p.q.filter(|s| !s.is_empty());

    let query = EntryQuery {
        feed_id: p.feed_id,
        since,
        until,
        q,
        limit,
        offset,
    };
    let conn = state.db.lock().unwrap();
    match db::query_entries(&conn, &query) {
        Ok((total, items)) => Json(json!({ "total": total, "items": items })).into_response(),
        Err(e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

/// Parse a caller-supplied RFC 3339 bound into normalized `...Z` form. `None`
/// input yields `Ok(None)`; unparseable input yields `Err`.
fn normalize_bound(input: Option<&str>) -> Result<Option<String>, ()> {
    match input {
        None => Ok(None),
        Some("") => Ok(None),
        Some(s) => parse_date(s).map(|d| Some(to_rfc3339_z(&d))).ok_or(()),
    }
}

fn valid_http_url(url: &str) -> bool {
    let rest = match url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))
    {
        Some(r) => r,
        None => return false,
    };
    // Require a non-empty authority component.
    let host = rest.split(['/', '?', '#']).next().unwrap_or("");
    !host.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_validation() {
        assert!(valid_http_url("http://a/b"));
        assert!(valid_http_url("https://example.com"));
        assert!(!valid_http_url("ftp://x"));
        assert!(!valid_http_url("not a url"));
        assert!(!valid_http_url("http://"));
    }

    #[test]
    fn bound_normalization() {
        assert_eq!(normalize_bound(None), Ok(None));
        assert_eq!(normalize_bound(Some("")), Ok(None));
        assert_eq!(
            normalize_bound(Some("2024-01-02T03:04:05-05:00")),
            Ok(Some("2024-01-02T08:04:05Z".to_string()))
        );
        assert_eq!(normalize_bound(Some("nope")), Err(()));
    }
}
