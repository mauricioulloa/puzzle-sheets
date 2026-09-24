//! Client for chess-puzzle-api, which owns the puzzle data.

use anyhow::{Context, Result};
use reqwest::StatusCode;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_secs(20);

/// Puzzles and solutions kept in memory, each. A puzzle never changes
/// between imports, so an entry is never stale; the cap only bounds memory,
/// at a few hundred bytes an entry.
const CACHE_CAPACITY: usize = 10_000;

/// Only the fields a worksheet uses.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Puzzle {
    pub id: String,
    pub position_fen: String,
    pub solver_color: String,
    pub themes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Solution {
    pub solution_san: Vec<String>,
}

#[derive(Deserialize)]
struct Batch {
    puzzles: Vec<Puzzle>,
}

/// What a worksheet asks the API for.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Filter {
    /// A Lichess theme name; none means any theme.
    pub theme: Option<&'static str>,
    pub rating_min: u32,
    pub rating_max: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum ChessApiError {
    /// No puzzle answers to this id.
    #[error("not found")]
    NotFound,
    /// The API is over its rate limit or stopped a slow search. Worth
    /// retrying, unlike everything else here.
    #[error("chess-puzzle-api is busy")]
    Busy { retry_after: Option<u64> },
    #[error("{0:#}")]
    Failed(#[from] anyhow::Error),
}

/// A map that stops growing at its capacity by forgetting an arbitrary
/// entry. Crude, but every entry is equally cheap to fetch again.
struct Cache<V> {
    entries: Mutex<HashMap<String, V>>,
}

impl<V: Clone> Cache<V> {
    fn new() -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
        }
    }

    fn get(&self, id: &str) -> Option<V> {
        self.lock().get(id).cloned()
    }

    fn insert(&self, id: &str, value: V) {
        let mut entries = self.lock();
        if entries.len() >= CACHE_CAPACITY
            && !entries.contains_key(id)
            && let Some(evicted) = entries.keys().next().cloned()
        {
            entries.remove(&evicted);
        }
        entries.insert(id.to_string(), value);
    }

    /// Nothing panics while holding the lock, and a map that did would still
    /// be whole, so a poisoned lock is used as it is.
    fn lock(&self) -> MutexGuard<'_, HashMap<String, V>> {
        self.entries.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

pub struct ChessApi {
    http: reqwest::Client,
    base_url: String,
    key: Option<String>,
    puzzles: Cache<Puzzle>,
    solutions: Cache<Solution>,
}

impl ChessApi {
    pub fn new(base_url: &str, key: Option<String>) -> Self {
        Self {
            // A sheet is a page someone is waiting on: better a clear error
            // than a browser spinning on a request that will never finish.
            http: reqwest::Client::builder()
                .timeout(TIMEOUT)
                .build()
                .expect("a client with a timeout always builds"),
            base_url: base_url.trim_end_matches('/').to_string(),
            key,
            puzzles: Cache::new(),
            solutions: Cache::new(),
        }
    }

    /// Up to `count` puzzles; none when the filter matches nothing. They are
    /// remembered, so the sheet they are picked for needs only solutions.
    pub async fn random(
        &self,
        filter: &Filter,
        count: usize,
    ) -> Result<Vec<Puzzle>, ChessApiError> {
        let mut query = vec![
            ("ratingMin", filter.rating_min.to_string()),
            ("ratingMax", filter.rating_max.to_string()),
            ("count", count.to_string()),
        ];
        if let Some(theme) = filter.theme {
            query.push(("themes", theme.to_string()));
        }
        match self.get::<Batch>("/v1/puzzles/random", &query).await {
            Ok(batch) => {
                for puzzle in &batch.puzzles {
                    self.puzzles.insert(&puzzle.id, puzzle.clone());
                }
                Ok(batch.puzzles)
            }
            // The API's 404 for a filter that matches nothing.
            Err(ChessApiError::NotFound) => Ok(Vec::new()),
            Err(err) => Err(err),
        }
    }

    pub async fn puzzle(&self, id: &str) -> Result<Puzzle, ChessApiError> {
        if let Some(puzzle) = self.puzzles.get(id) {
            return Ok(puzzle);
        }
        let puzzle: Puzzle = self.get(&format!("/v1/puzzles/{id}"), &[]).await?;
        self.puzzles.insert(id, puzzle.clone());
        Ok(puzzle)
    }

    pub async fn solution(&self, id: &str) -> Result<Solution, ChessApiError> {
        if let Some(solution) = self.solutions.get(id) {
            return Ok(solution);
        }
        let solution: Solution = self.get(&format!("/v1/puzzles/{id}/solution"), &[]).await?;
        self.solutions.insert(id, solution.clone());
        Ok(solution)
    }

    async fn get<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<T, ChessApiError> {
        let mut request = self
            .http
            .get(format!("{}{path}", self.base_url))
            .query(query);
        if let Some(key) = &self.key {
            request = request.bearer_auth(key);
        }
        let response = request
            .send()
            .await
            .with_context(|| format!("calling chess-puzzle-api {path}"))?;

        let status = response.status();
        if status == StatusCode::NOT_FOUND {
            return Err(ChessApiError::NotFound);
        }
        if status == StatusCode::TOO_MANY_REQUESTS || status == StatusCode::SERVICE_UNAVAILABLE {
            let retry_after = response
                .headers()
                .get(reqwest::header::RETRY_AFTER)
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.parse().ok());
            tracing::warn!("chess-puzzle-api {path} returned {status}");
            return Err(ChessApiError::Busy { retry_after });
        }
        // Everything sent here has been validated first, so a 400 means the
        // two services disagree, not that the visitor made a mistake.
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(
                anyhow::anyhow!("chess-puzzle-api {path} returned {status}: {body}").into(),
            );
        }
        Ok(response
            .json()
            .await
            .with_context(|| format!("decoding chess-puzzle-api {path}"))?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cache_stops_at_its_capacity() {
        let cache = Cache::new();
        for n in 0..CACHE_CAPACITY + 5 {
            cache.insert(&n.to_string(), n);
        }
        assert_eq!(cache.lock().len(), CACHE_CAPACITY);
        let last = (CACHE_CAPACITY + 4).to_string();
        assert_eq!(
            cache.get(&last),
            Some(CACHE_CAPACITY + 4),
            "the newest stays"
        );
    }
}
