//! The REST API.
//!
//! Errors are `{"error": "<message>"}` with a 4xx/5xx status, uniformly —
//! including for routes that do not exist and bodies that do not parse, which
//! is why the extractors below are fallible rather than plain `Json<T>`.

use std::collections::HashMap;

use axum::Json;
use axum::Router;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use chrono::Utc;
use feedhub_core::api::{
    AddFeedRequest, DEFAULT_LIMIT, EntriesPage, ErrorBody, Feed, MAX_LIMIT, RefreshResult,
};
use feedhub_core::parse_rfc3339;
use serde_json::json;
use url::Url;

use crate::db::{self, EntryQuery, InsertFeedError};
use crate::refresh;
use crate::state::SharedState;

/// An error response: a status and a message, rendered as `{"error": ...}`.
pub struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn new(status: StatusCode, message: impl Into<String>) -> Self {
        ApiError {
            status,
            message: message.into(),
        }
    }

    fn not_found(message: impl Into<String>) -> Self {
        ApiError::new(StatusCode::NOT_FOUND, message)
    }

    fn unprocessable(message: impl Into<String>) -> Self {
        ApiError::new(StatusCode::UNPROCESSABLE_ENTITY, message)
    }

    fn internal(error: impl std::fmt::Display) -> Self {
        ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("internal error: {error}"),
        )
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorBody {
                error: self.message,
            }),
        )
            .into_response()
    }
}

type ApiResult<T> = Result<T, ApiError>;

pub fn router(state: SharedState) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/feeds", get(list_feeds).post(add_feed))
        .route("/api/feeds/{id}", get(get_feed).delete(remove_feed))
        .route("/api/feeds/{id}/refresh", post(refresh_one))
        .route("/api/refresh", post(refresh_every))
        .route("/api/entries", get(list_entries))
        .fallback(unknown_route)
        .with_state(state)
}

async fn unknown_route() -> ApiError {
    ApiError::not_found("no such endpoint")
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({"status": "ok", "version": env!("CARGO_PKG_VERSION")}))
}

async fn list_feeds(State(state): State<SharedState>) -> ApiResult<Json<Vec<Feed>>> {
    let conn = state.db.lock().expect("db mutex poisoned");
    db::list_feeds(&conn).map(Json).map_err(ApiError::internal)
}

async fn add_feed(
    State(state): State<SharedState>,
    body: Result<Json<AddFeedRequest>, JsonRejection>,
) -> ApiResult<(StatusCode, Json<Feed>)> {
    let Json(request) = body
        .map_err(|_| ApiError::unprocessable("body must be a JSON object with a \"url\" string"))?;
    validate_feed_url(&request.url)?;

    let conn = state.db.lock().expect("db mutex poisoned");
    match db::insert_feed(&conn, &request.url, Utc::now()) {
        Ok(feed) => Ok((StatusCode::CREATED, Json(feed))),
        Err(InsertFeedError::Duplicate) => Err(ApiError::new(
            StatusCode::CONFLICT,
            format!("feed already registered: {}", request.url),
        )),
        Err(InsertFeedError::Db(e)) => Err(ApiError::internal(e)),
    }
}

/// Only absolute http(s) URLs with a host are feeds we can fetch.
fn validate_feed_url(url: &str) -> ApiResult<()> {
    let parsed =
        Url::parse(url).map_err(|_| ApiError::unprocessable(format!("not a valid URL: {url}")))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(ApiError::unprocessable(format!(
            "URL scheme must be http or https, not {}",
            parsed.scheme()
        )));
    }
    if parsed.host().is_none() {
        return Err(ApiError::unprocessable(format!("URL has no host: {url}")));
    }
    Ok(())
}

async fn get_feed(State(state): State<SharedState>, Path(id): Path<i64>) -> ApiResult<Json<Feed>> {
    let conn = state.db.lock().expect("db mutex poisoned");
    db::get_feed(&conn, id)
        .map_err(ApiError::internal)?
        .map(Json)
        .ok_or_else(|| ApiError::not_found(format!("no feed with id {id}")))
}

async fn remove_feed(
    State(state): State<SharedState>,
    Path(id): Path<i64>,
) -> ApiResult<StatusCode> {
    let conn = state.db.lock().expect("db mutex poisoned");
    // Entries go with it, by way of ON DELETE CASCADE.
    match db::delete_feed(&conn, id).map_err(ApiError::internal)? {
        true => Ok(StatusCode::NO_CONTENT),
        false => Err(ApiError::not_found(format!("no feed with id {id}"))),
    }
}

/// A fetch that fails is still a successful request: the outcome is reported in
/// the body's `status` field, not as an HTTP error.
async fn refresh_one(
    State(state): State<SharedState>,
    Path(id): Path<i64>,
) -> ApiResult<Json<RefreshResult>> {
    refresh::refresh_feed(&state, id)
        .await
        .map(Json)
        .ok_or_else(|| ApiError::not_found(format!("no feed with id {id}")))
}

async fn refresh_every(State(state): State<SharedState>) -> Json<Vec<RefreshResult>> {
    Json(refresh::refresh_all(&state).await)
}

async fn list_entries(
    State(state): State<SharedState>,
    Query(params): Query<HashMap<String, String>>,
) -> ApiResult<Json<EntriesPage>> {
    let query = parse_entry_query(&params)?;
    let conn = state.db.lock().expect("db mutex poisoned");
    db::query_entries(&conn, &query)
        .map(Json)
        .map_err(ApiError::internal)
}

fn parse_entry_query(params: &HashMap<String, String>) -> ApiResult<EntryQuery> {
    let integer = |name: &str| -> ApiResult<Option<i64>> {
        params
            .get(name)
            .map(|raw| {
                raw.parse::<i64>()
                    .map_err(|_| ApiError::unprocessable(format!("{name} must be an integer")))
            })
            .transpose()
    };

    // Bounds are normalized to the storage format here, so the comparison in SQL
    // is between two UTC instants regardless of the offset the caller used.
    let instant = |name: &str| -> ApiResult<Option<String>> {
        params
            .get(name)
            .map(|raw| {
                parse_rfc3339(raw)
                    .map(feedhub_core::format_utc)
                    .ok_or_else(|| {
                        ApiError::unprocessable(format!("{name} must be an RFC 3339 timestamp"))
                    })
            })
            .transpose()
    };

    let limit = match integer("limit")? {
        Some(limit) if limit < 0 => {
            return Err(ApiError::unprocessable("limit must not be negative"));
        }
        Some(limit) => limit.min(MAX_LIMIT),
        None => DEFAULT_LIMIT,
    };
    let offset = match integer("offset")? {
        Some(offset) if offset < 0 => {
            return Err(ApiError::unprocessable("offset must not be negative"));
        }
        Some(offset) => offset,
        None => 0,
    };

    Ok(EntryQuery {
        feed_id: integer("feed_id")?,
        since: instant("since")?,
        until: instant("until")?,
        search: params.get("q").filter(|q| !q.is_empty()).cloned(),
        limit,
        offset,
    })
}
