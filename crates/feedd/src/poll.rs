//! Background polling.
//!
//! The loop is deliberately thin: it reuses the exact same `refresh_feed` path
//! the REST endpoints use, so there is no second code path to keep correct.
//! [`poll_once`] is public precisely so a test can tick the loop deterministically
//! instead of racing a timer — the poller is the one component that runs
//! unattended, so "we never test it" is not an option.

use std::sync::Arc;
use std::time::Duration;

use feedhub_core::model::RefreshResult;

use crate::api::refresh_all;
use crate::fetch::Fetcher;
use crate::store::Store;

/// Refresh every feed exactly once.
pub async fn poll_once(store: &Store, fetcher: &Fetcher) -> Vec<RefreshResult> {
    match refresh_all(store, fetcher).await {
        Ok(results) => results,
        Err(e) => {
            // Listing the feeds failed. Log and yield nothing; the next tick
            // tries again. The loop must not die.
            tracing::error!(error = %e, "poll could not list feeds");
            Vec::new()
        }
    }
}

/// Spawn the poll loop. A zero interval disables polling entirely, and this
/// returns `None`.
pub fn spawn(store: Arc<Store>, fetcher: Arc<Fetcher>, interval: Duration) -> Option<tokio::task::JoinHandle<()>> {
    if interval.is_zero() {
        tracing::info!("background polling disabled (--poll-interval 0)");
        return None;
    }

    tracing::info!(interval_secs = interval.as_secs(), "background polling enabled");
    Some(tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        // The first tick fires immediately; skip it so startup isn't a
        // thundering herd against every registered origin.
        ticker.tick().await;
        loop {
            ticker.tick().await;
            let results = poll_once(&store, &fetcher).await;
            let failed = results.iter().filter(|r| r.status == "error").count();
            let new: usize = results.iter().map(|r| r.new_entries).sum();
            tracing::info!(
                feeds = results.len(),
                new_entries = new,
                failed,
                "poll complete"
            );
        }
    }))
}
