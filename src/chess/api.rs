//! Client for chess-puzzle-api, which owns the puzzle data.

use anyhow::{Context, Result};
use reqwest::StatusCode;
use serde::Deserialize;
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_secs(20);

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

#[derive(Deserialize)]
struct ErrorBody {
    message: String,
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
    /// The API understood the request and refused it; the message says why.
    #[error("{0}")]
    Rejected(String),
    #[error("{0:#}")]
    Failed(#[from] anyhow::Error),
}

pub struct ChessApi {
    http: reqwest::Client,
    base_url: String,
    key: Option<String>,
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
        }
    }

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
        let batch: Batch = self.get("/v1/puzzles/random", &query).await?;
        Ok(batch.puzzles)
    }

    pub async fn puzzle(&self, id: &str) -> Result<Puzzle, ChessApiError> {
        self.get(&format!("/v1/puzzles/{id}"), &[]).await
    }

    pub async fn solution(&self, id: &str) -> Result<Solution, ChessApiError> {
        self.get(&format!("/v1/puzzles/{id}/solution"), &[]).await
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
        if status == StatusCode::BAD_REQUEST || status == StatusCode::NOT_FOUND {
            let body: ErrorBody = response.json().await.context("reading the API's error")?;
            return Err(ChessApiError::Rejected(body.message));
        }
        if !status.is_success() {
            return Err(anyhow::anyhow!("chess-puzzle-api {path} returned {status}").into());
        }
        Ok(response
            .json()
            .await
            .with_context(|| format!("decoding chess-puzzle-api {path}"))?)
    }
}
