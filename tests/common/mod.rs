//! A stand-in for chess-puzzle-api that serves two known puzzles and records
//! what it was asked, shared by the integration suites.

use axum::extract::{Path, RawQuery, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use puzzle_sheets::chess::api::ChessApi;
use puzzle_sheets::web::routes::{self, AppState};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

pub const PUBLIC_URL: &str = "https://sheets.example";

/// Every request the stand-in answered, in order: a pick as its query
/// string, a lookup as its path.
pub type Seen = Arc<Mutex<Vec<String>>>;

fn puzzle(id: &str) -> Option<Value> {
    match id {
        "00008" => Some(json!({
            "id": "00008",
            "positionFen": "r6k/pp2r2p/4Rp1Q/3p4/8/1N1P2b1/PqP3PP/7K w - - 0 25",
            "solverColor": "white",
            "themes": ["middlegame", "crushing", "long", "hangingPiece"],
        })),
        "000Zo" => Some(json!({
            "id": "000Zo",
            "positionFen": "3r2k1/5ppp/8/8/8/8/5PPP/6K1 b - - 0 1",
            "solverColor": "black",
            "themes": ["mate", "mateIn1", "backRankMate"],
        })),
        _ => None,
    }
}

fn solution(id: &str) -> Value {
    match id {
        "00008" => json!({"solutionSan": ["Rxe7", "Qb1+", "Nc1", "Qxc1+", "Qxc1"]}),
        _ => json!({"solutionSan": ["Rd1#"]}),
    }
}

/// The theme and the id the stand-in answers `429` to, the way the API does
/// over its rate limit.
pub const BUSY_THEME: &str = "zugzwang";
pub const BUSY_ID: &str = "busy1";
pub const BUSY_RETRY_AFTER: u64 = 17;

fn busy() -> Response {
    (
        StatusCode::TOO_MANY_REQUESTS,
        [("retry-after", BUSY_RETRY_AFTER.to_string())],
        Json(json!({"error": "rate_limited", "message": "Rate limit exceeded."})),
    )
        .into_response()
}

async fn stub_random(State(seen): State<Seen>, RawQuery(query): RawQuery) -> Response {
    let query = query.unwrap_or_default();
    if query.contains(&format!("themes={BUSY_THEME}")) {
        return busy();
    }
    let count: usize = query
        .split('&')
        .find_map(|pair| pair.strip_prefix("count="))
        .and_then(|value| value.parse().ok())
        .unwrap_or(1);
    seen.lock().unwrap().push(query);
    let puzzles: Vec<Value> = ["00008", "000Zo"]
        .iter()
        .cycle()
        .take(count)
        .filter_map(|id| puzzle(id))
        .collect();
    Json(json!({"count": puzzles.len(), "puzzles": puzzles})).into_response()
}

fn not_found(id: &str) -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(json!({"error": "not_found", "message": format!("No puzzle with id `{id}`.")})),
    )
        .into_response()
}

async fn stub_puzzle(State(seen): State<Seen>, Path(id): Path<String>) -> Response {
    seen.lock().unwrap().push(format!("/v1/puzzles/{id}"));
    if id == BUSY_ID {
        return busy();
    }
    match puzzle(&id) {
        Some(body) => Json(body).into_response(),
        None => not_found(&id),
    }
}

async fn stub_solution(State(seen): State<Seen>, Path(id): Path<String>) -> Response {
    seen.lock()
        .unwrap()
        .push(format!("/v1/puzzles/{id}/solution"));
    if id == BUSY_ID {
        return busy();
    }
    match puzzle(&id) {
        Some(_) => Json(solution(&id)).into_response(),
        None => not_found(&id),
    }
}

/// Starts the stand-in on a free port and returns its URL.
async fn chess_api(seen: Seen) -> String {
    let app = Router::new()
        .route("/v1/puzzles/random", get(stub_random))
        .route("/v1/puzzles/{id}", get(stub_puzzle))
        .route("/v1/puzzles/{id}/solution", get(stub_solution))
        .with_state(seen);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    url
}

/// The app wired to a fresh stand-in, and what the stand-in is asked.
pub async fn app() -> (Router, Seen) {
    let seen = Seen::default();
    let url = chess_api(Arc::clone(&seen)).await;
    (app_against(&url), seen)
}

pub fn app_against(chess_api_url: &str) -> Router {
    let state = Arc::new(AppState {
        api: ChessApi::new(chess_api_url, None),
        public_url: PUBLIC_URL.to_string(),
    });
    routes::router(state, &[])
}
