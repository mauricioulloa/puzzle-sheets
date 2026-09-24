//! The HTTP and MCP surfaces, against a stand-in for chess-puzzle-api that
//! serves two known puzzles and records what it was asked.

use axum::body::Body;
use axum::extract::{Path, RawQuery, State};
use axum::http::{Request, StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use http_body_util::BodyExt;
use puzzle_sheets::chess::api::ChessApi;
use puzzle_sheets::web::routes::{self, AppState};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
use tower::ServiceExt;

const PUBLIC_URL: &str = "https://sheets.example";

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

type Seen = Arc<Mutex<Vec<String>>>;

async fn stub_random(State(seen): State<Seen>, RawQuery(query): RawQuery) -> Json<Value> {
    let query = query.unwrap_or_default();
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
    Json(json!({"count": puzzles.len(), "puzzles": puzzles}))
}

fn not_found(id: &str) -> axum::response::Response {
    (
        StatusCode::NOT_FOUND,
        Json(json!({"error": "not_found", "message": format!("No puzzle with id `{id}`.")})),
    )
        .into_response()
}

async fn stub_puzzle(Path(id): Path<String>) -> axum::response::Response {
    match puzzle(&id) {
        Some(body) => Json(body).into_response(),
        None => not_found(&id),
    }
}

async fn stub_solution(Path(id): Path<String>) -> axum::response::Response {
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

async fn app() -> (Router, Seen) {
    let seen = Seen::default();
    let url = chess_api(Arc::clone(&seen)).await;
    (app_against(&url), seen)
}

fn app_against(chess_api_url: &str) -> Router {
    let state = Arc::new(AppState {
        api: ChessApi::new(chess_api_url, None),
        public_url: PUBLIC_URL.to_string(),
    });
    routes::router(state, &[])
}

async fn get_page(app: &Router, uri: &str) -> (StatusCode, String, Option<String>) {
    let response = app
        .clone()
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let location = response
        .headers()
        .get(header::LOCATION)
        .map(|value| value.to_str().unwrap().to_string());
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        String::from_utf8_lossy(&bytes).into_owned(),
        location,
    )
}

#[tokio::test]
async fn a_level_and_theme_become_a_permanent_sheet_address() {
    let (app, seen) = app().await;
    let (status, _, location) = get_page(
        &app,
        "/sheet/new?level=beginner&theme=mateIn1&count=2&lang=en&answers=none",
    )
    .await;

    assert_eq!(status, StatusCode::SEE_OTHER);
    let location = location.expect("redirect");
    assert_eq!(
        location,
        "/sheet?ids=00008%2C000Zo&level=beginner&lang=en&answers=none&theme=mateIn1"
    );

    let query = seen.lock().unwrap().pop().expect("the API was asked");
    for expected in ["themes=mateIn1", "ratingMin=0", "ratingMax=999", "count=2"] {
        assert!(query.contains(expected), "{expected} missing from {query}");
    }
}

#[tokio::test]
async fn any_theme_asks_for_none_and_a_level_is_only_a_rating_band() {
    let (app, seen) = app().await;
    get_page(&app, "/sheet/new?level=advanced").await;

    let query = seen.lock().unwrap().pop().expect("the API was asked");
    assert!(!query.contains("themes="), "{query}");
    assert!(!query.contains("maxPieces"), "{query}");
    assert!(query.contains("ratingMin=1800"), "{query}");
    assert!(query.contains("ratingMax=2199"), "{query}");
}

#[tokio::test]
async fn each_puzzle_says_who_moves_what_to_find_and_its_fen() {
    let (app, _) = app().await;
    let (status, html, _) = get_page(
        &app,
        "/sheet?ids=00008,000Zo&lang=en&level=novice&theme=fork",
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert!(html.contains("Difficulty: Novice (1000–1399) · Theme: Fork"));
    assert!(html.contains("Win decisively"));
    assert!(html.contains("Mate in 1"));
    // Black to move: the marker is filled and sits at the top.
    assert!(html.contains("y=\"0\" width=\"22\" height=\"22\" fill=\"#000\""));
    assert!(html.contains("r6k/pp2r2p/4Rp1Q/3p4/8/1N1P2b1/PqP3PP/7K w - - 0 25"));
    assert!(html.contains("3r2k1/5ppp/8/8/8/8/5PPP/6K1 b - - 0 1"));
}

#[tokio::test]
async fn solutions_go_on_their_own_page_unless_left_out() {
    let (app, _) = app().await;
    let (_, html, _) = get_page(&app, "/sheet?ids=00008,000Zo&lang=en").await;
    let answers = html
        .find("class=\"page answers\"")
        .expect("a solutions page by default");
    assert!(html.find("3r2k1/5ppp").unwrap() < answers, "puzzles first");
    assert!(html.find("Win decisively: 1. Rxe7").unwrap() > answers);

    let (_, student, _) = get_page(&app, "/sheet?ids=00008,000Zo&lang=en&answers=none").await;
    assert!(
        !student.contains("Rxe7"),
        "a student sheet carries no answers"
    );

    let (status, _, _) = get_page(&app, "/sheet?ids=00008&answers=footer").await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "footer solutions are gone");
}

#[tokio::test]
async fn spanish_sheets_read_in_spanish() {
    let (app, _) = app().await;
    let (_, html, _) = get_page(&app, "/sheet?ids=000Zo&lang=es").await;

    assert!(html.contains("Mate en 1"));
    assert!(
        html.contains("Dificultad: Inicial"),
        "novice is the default level"
    );
    assert!(html.contains("1... Td1#"), "Spanish piece letters");
    assert!(html.contains("lang=\"es\""));
}

#[tokio::test]
async fn bad_requests_explain_themselves() {
    let (app, seen) = app().await;

    let (status, html, _) = get_page(&app, "/sheet?ids=nope1").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        html.contains("No puzzle with id"),
        "the API's reason is passed on"
    );

    for uri in [
        "/sheet?ids=../v1/stats",
        "/sheet?ids=00008&answers=sideways",
        "/sheet/new?level=grandmaster",
        "/sheet/new?theme=chessboxing",
        "/sheet/new?count=13",
    ] {
        let (status, _, _) = get_page(&app, uri).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{uri}");
    }
    assert!(
        seen.lock().unwrap().is_empty(),
        "invalid requests never reach the API"
    );
}

#[tokio::test]
async fn an_unreachable_api_is_a_bad_gateway() {
    // Nothing listens on port 9 (discard) on a test machine.
    let app = app_against("http://127.0.0.1:9");
    let (status, _, _) = get_page(&app, "/sheet?ids=00008").await;
    assert_eq!(status, StatusCode::BAD_GATEWAY);
}

#[tokio::test]
async fn the_landing_page_speaks_the_browsers_language() {
    let (app, _) = app().await;
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/")
                .header(header::ACCEPT_LANGUAGE, "es-CL,es;q=0.9")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let html = String::from_utf8_lossy(&response.into_body().collect().await.unwrap().to_bytes())
        .into_owned();
    assert!(html.contains("Hojas de ejercicios de ajedrez"));
}

async fn rpc(app: &Router, method: &str, params: Value) -> Value {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mcp")
                .header("content-type", "application/json")
                .header("accept", "application/json, text/event-stream")
                // rmcp rejects a request without Host as DNS-rebinding defence.
                .header("host", "localhost")
                .body(Body::from(
                    json!({"jsonrpc": "2.0", "id": 1, "method": method, "params": params})
                        .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

fn structured(response: &Value) -> &Value {
    &response["result"]["structuredContent"]
}

#[tokio::test]
async fn agents_can_list_options_and_make_a_sheet() {
    let (app, _) = app().await;

    let options = rpc(
        &app,
        "tools/call",
        json!({"name": "list_options", "arguments": {"lang": "en"}}),
    )
    .await;
    let options = structured(&options);
    assert_eq!(
        options["levels"].as_array().expect("levels").len(),
        puzzle_sheets::chess::options::LEVELS.len()
    );
    assert_eq!(
        options["themes"].as_array().expect("themes").len(),
        puzzle_sheets::chess::options::THEMES.len()
    );

    let created = rpc(
        &app,
        "tools/call",
        json!({"name": "create_worksheet", "arguments": {"level": "beginner", "theme": "fork", "count": 2, "answers": "none"}}),
    )
    .await;
    let url = structured(&created)["worksheet_url"].as_str().expect("url");
    assert!(
        url.starts_with(&format!("{PUBLIC_URL}/sheet?ids=")),
        "{url}"
    );
    assert!(url.contains("answers=none"));
}

#[tokio::test]
async fn agents_are_told_what_went_wrong() {
    let (app, _) = app().await;
    let response = rpc(
        &app,
        "tools/call",
        json!({"name": "create_worksheet", "arguments": {"theme": "chessboxing"}}),
    )
    .await;
    let message = response["error"]["message"].as_str().expect("an error");
    assert!(
        message.contains("zugzwang"),
        "it lists the themes that do exist"
    );
}
