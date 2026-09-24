//! The HTTP surface: the form, the sheet and its errors, against a stand-in
//! for chess-puzzle-api.

mod common;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use common::{PUBLIC_URL, app, app_against};
use http_body_util::BodyExt;
use tower::ServiceExt;

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

#[tokio::test]
async fn llms_txt_links_to_the_public_address() {
    let (app, _) = app().await;
    let (status, text, _) = get_page(&app, "/llms.txt").await;
    assert_eq!(status, StatusCode::OK);
    assert!(text.starts_with("# puzzle-sheets"));
    assert!(
        text.contains(&format!("{PUBLIC_URL}/mcp")),
        "an agent cannot resolve a relative link"
    );
}
