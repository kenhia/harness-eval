//! The REST API.

use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use feedhub_core::datetime::{self, parse_rfc3339};
use feedhub_core::model::RefreshResult;
use serde::Deserialize;
use serde_json::json;

use crate::fetch::{Fetched, Fetcher};
use crate::store::{EntryQuery, FetchState, Store, StoreError};

pub const DEFAULT_LIMIT: i64 = 50;
pub const MAX_LIMIT: i64 = 500;

#[derive(Clone)]
pub struct AppState {
    pub store: Arc<Store>,
    pub fetcher: Arc<Fetcher>,
}

/// An error in the pinned `{"error": "..."}` shape.
#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }

    fn not_found(what: impl std::fmt::Display) -> Self {
        Self::new(StatusCode::NOT_FOUND, format!("{what} not found"))
    }

    fn unprocessable(message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNPROCESSABLE_ENTITY, message)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(json!({ "error": self.message }))).into_response()
    }
}

impl From<StoreError> for ApiError {
    fn from(e: StoreError) -> Self {
        match e {
            StoreError::DuplicateUrl => ApiError::new(StatusCode::CONFLICT, e.to_string()),
            StoreError::Sqlite(e) => {
                // The client can't act on SQLite internals; log them, return a
                // generic 500.
                tracing::error!(error = %e, "storage failure");
                ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, "storage failure")
            }
        }
    }
}

type ApiResult<T> = Result<T, ApiError>;

// ------------------------------------------------------------------ health

async fn health() -> impl IntoResponse {
    Json(json!({ "status": "ok", "version": env!("CARGO_PKG_VERSION") }))
}

// ------------------------------------------------------------------- feeds

#[derive(Debug, Deserialize)]
struct AddFeedBody {
    url: String,
}

async fn add_feed(
    State(state): State<AppState>,
    body: Result<Json<AddFeedBody>, JsonRejection>,
) -> ApiResult<Response> {
    // Handle the rejection ourselves so a bad body still gets the pinned error
    // envelope instead of axum's default text.
    let Json(body) = body.map_err(|e| ApiError::unprocessable(e.body_text()))?;

    crate::fetch::validate_feed_url(&body.url).map_err(ApiError::unprocessable)?;

    let feed = state.store.add_feed(&body.url)?;
    Ok((StatusCode::CREATED, Json(feed)).into_response())
}

async fn list_feeds(State(state): State<AppState>) -> ApiResult<Response> {
    Ok(Json(state.store.list_feeds()?).into_response())
}

async fn get_feed(State(state): State<AppState>, Path(id): Path<i64>) -> ApiResult<Response> {
    let feed = state
        .store
        .get_feed(id)?
        .ok_or_else(|| ApiError::not_found(format!("feed {id}")))?;
    Ok(Json(feed).into_response())
}

async fn remove_feed(State(state): State<AppState>, Path(id): Path<i64>) -> ApiResult<Response> {
    if state.store.delete_feed(id)? {
        Ok(StatusCode::NO_CONTENT.into_response())
    } else {
        Err(ApiError::not_found(format!("feed {id}")))
    }
}

// ----------------------------------------------------------------- refresh

/// Refresh one feed: fetch now, then apply.
///
/// Note the shape: the store lock is taken and released inside each `store.*`
/// call, and the network round-trip happens between them, never inside one.
/// `MutexGuard: !Send` means the compiler rejects any refactor that gets this
/// wrong. See the `store` module docs.
pub async fn refresh_feed(store: &Store, fetcher: &Fetcher, target: &FetchState) -> RefreshResult {
    let now = datetime::now();

    match fetcher.fetch(target).await {
        Ok(Fetched::NotModified) => match store.apply_not_modified(target.id, now) {
            Ok(()) => RefreshResult::ok(target.id, 0, 0, true),
            Err(e) => RefreshResult::error(target.id, e.to_string()),
        },
        Ok(Fetched::Body(success)) => match store.apply_success(target.id, &success, now) {
            Ok(counts) => RefreshResult::ok(target.id, counts.new, counts.updated, false),
            Err(e) => RefreshResult::error(target.id, e.to_string()),
        },
        Err(e) => {
            // A failure is recorded against this feed and goes no further: it
            // must never disturb another feed or take the server down.
            let message = e.to_string();
            tracing::warn!(feed_id = target.id, url = %target.url, error = %message, "refresh failed");
            if let Err(e) = store.apply_error(target.id, &message, now) {
                tracing::error!(feed_id = target.id, error = %e, "could not record fetch error");
            }
            RefreshResult::error(target.id, message)
        }
    }
}

async fn refresh_one(State(state): State<AppState>, Path(id): Path<i64>) -> ApiResult<Response> {
    let target = state
        .store
        .fetch_state(id)?
        .ok_or_else(|| ApiError::not_found(format!("feed {id}")))?;

    let result = refresh_feed(&state.store, &state.fetcher, &target).await;
    Ok(Json(result).into_response())
}

/// Refresh every feed, sequentially.
///
/// Sequential on purpose: feeds are few, and it keeps one slow origin from
/// contending for the store lock with N-1 others. Each feed's result is
/// independent, so one failure never truncates the array.
pub async fn refresh_all(
    store: &Store,
    fetcher: &Fetcher,
) -> Result<Vec<RefreshResult>, StoreError> {
    let targets = store.all_fetch_states()?;
    let mut results = Vec::with_capacity(targets.len());
    for target in &targets {
        results.push(refresh_feed(store, fetcher, target).await);
    }
    Ok(results)
}

async fn refresh_all_handler(State(state): State<AppState>) -> ApiResult<Response> {
    let results = refresh_all(&state.store, &state.fetcher).await?;
    Ok(Json(results).into_response())
}

// ----------------------------------------------------------------- entries

/// Parse `GET /api/entries` parameters.
///
/// Done by hand from a string map rather than with a typed `Query<T>` so that a
/// bad value produces the pinned `{"error": ...}` envelope with a 422, instead
/// of axum's default rejection body.
fn parse_entry_query(params: &HashMap<String, String>) -> ApiResult<EntryQuery> {
    let int = |name: &str| -> ApiResult<Option<i64>> {
        match params.get(name) {
            None => Ok(None),
            Some(raw) => raw.parse::<i64>().map(Some).map_err(|_| {
                ApiError::unprocessable(format!("{name} must be an integer, got {raw:?}"))
            }),
        }
    };

    let instant = |name: &str| -> ApiResult<Option<i64>> {
        match params.get(name) {
            None => Ok(None),
            Some(raw) => parse_rfc3339(raw)
                .map(|dt| Some(datetime::to_millis(dt)))
                .ok_or_else(|| {
                    ApiError::unprocessable(format!(
                        "{name} must be an RFC 3339 instant, got {raw:?}"
                    ))
                }),
        }
    };

    let limit = match int("limit")? {
        // Above the ceiling we clamp rather than reject: the spec pins a max,
        // and silently giving less than asked is friendlier than a 422.
        Some(n) if n > MAX_LIMIT => MAX_LIMIT,
        Some(n) if n < 0 => {
            return Err(ApiError::unprocessable("limit must not be negative"));
        }
        Some(n) => n,
        None => DEFAULT_LIMIT,
    };
    let offset = match int("offset")? {
        Some(n) if n < 0 => {
            return Err(ApiError::unprocessable("offset must not be negative"));
        }
        Some(n) => n,
        None => 0,
    };

    Ok(EntryQuery {
        feed_id: int("feed_id")?,
        since: instant("since")?,
        until: instant("until")?,
        q: params.get("q").filter(|s| !s.is_empty()).cloned(),
        limit,
        offset,
    })
}

async fn list_entries(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> ApiResult<Response> {
    let query = parse_entry_query(&params)?;
    Ok(Json(state.store.query_entries(&query)?).into_response())
}

// ------------------------------------------------------------------ router

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/feeds", get(list_feeds).post(add_feed))
        .route("/api/feeds/{id}", get(get_feed))
        .route("/api/feeds/{id}", delete(remove_feed))
        .route("/api/feeds/{id}/refresh", post(refresh_one))
        .route("/api/refresh", post(refresh_all_handler))
        .route("/api/entries", get(list_entries))
        .fallback(|| async {
            ApiError::new(StatusCode::NOT_FOUND, "no such endpoint").into_response()
        })
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect()
    }

    #[test]
    fn defaults_are_limit_50_offset_0_and_no_filters() {
        let q = parse_entry_query(&params(&[])).unwrap();
        assert_eq!(q.limit, DEFAULT_LIMIT);
        assert_eq!(q.offset, 0);
        assert_eq!(q.feed_id, None);
        assert_eq!(q.since, None);
        assert_eq!(q.until, None);
        assert_eq!(q.q, None);
    }

    #[test]
    fn limit_is_clamped_to_the_maximum() {
        let q = parse_entry_query(&params(&[("limit", "100000")])).unwrap();
        assert_eq!(q.limit, MAX_LIMIT);
    }

    #[test]
    fn bounds_parse_from_any_offset_into_utc_millis() {
        let q = parse_entry_query(&params(&[
            ("since", "2020-01-01T00:00:00Z"),
            ("until", "2020-01-01T05:30:00+05:30"),
        ]))
        .unwrap();
        // Both name the same instant, expressed differently.
        assert_eq!(q.since, q.until);
    }

    #[test]
    fn fractional_second_bounds_survive_as_millis() {
        let q = parse_entry_query(&params(&[("since", "2020-01-01T00:00:00.500Z")])).unwrap();
        let whole = parse_entry_query(&params(&[("since", "2020-01-01T00:00:00Z")])).unwrap();
        assert_eq!(
            q.since.unwrap() - whole.since.unwrap(),
            500,
            "sub-second precision must reach the query, not be rounded away"
        );
    }

    #[test]
    fn bad_values_are_422_not_a_panic() {
        for bad in [
            vec![("limit", "abc")],
            vec![("offset", "1.5")],
            vec![("feed_id", "many")],
            vec![("since", "yesterday")],
            vec![("until", "2020-13-45")],
            vec![("limit", "-1")],
            vec![("offset", "-1")],
        ] {
            let err = parse_entry_query(&params(&bad)).unwrap_err();
            assert_eq!(
                err.status,
                StatusCode::UNPROCESSABLE_ENTITY,
                "{bad:?} should be unprocessable"
            );
        }
    }

    #[test]
    fn an_empty_q_is_not_a_filter() {
        let q = parse_entry_query(&params(&[("q", "")])).unwrap();
        assert_eq!(q.q, None, "?q= must not match nothing");
    }
}
