//! The HTTP surface: the form, the sheet and its errors, against a stand-in
//! for chess-puzzle-api.

mod common;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use common::{BUSY_ID, BUSY_RETRY_AFTER, PUBLIC_URL, app, app_against};
use http_body_util::BodyExt;
use puzzle_sheets::sheet::PRINT_SCRIPT;
use puzzle_sheets::web::routes::CONTENT_SECURITY_POLICY;
use sha2::{Digest, Sha256};
use tower::ServiceExt;

struct Answer {
    status: StatusCode,
    content_type: String,
    retry_after: Option<String>,
    cache_control: Option<String>,
    /// Every Vary value; compression adds its own.
    vary: Vec<String>,
    body: String,
}

async fn request(app: &Router, uri: &str, accept_language: Option<&str>) -> Answer {
    let mut builder = Request::builder().uri(uri);
    if let Some(value) = accept_language {
        builder = builder.header(header::ACCEPT_LANGUAGE, value);
    }
    let response = app
        .clone()
        .oneshot(builder.body(Body::empty()).unwrap())
        .await
        .unwrap();
    let header_text = |name| {
        response
            .headers()
            .get(name)
            .map(|value: &axum::http::HeaderValue| value.to_str().unwrap().to_string())
    };
    let status = response.status();
    let content_type = header_text(header::CONTENT_TYPE).unwrap_or_default();
    let retry_after = header_text(header::RETRY_AFTER);
    let cache_control = header_text(header::CACHE_CONTROL);
    let vary: Vec<String> = response
        .headers()
        .get_all(header::VARY)
        .iter()
        .map(|value| value.to_str().unwrap().to_ascii_lowercase())
        .collect();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    Answer {
        status,
        content_type,
        retry_after,
        cache_control,
        vary,
        body: String::from_utf8_lossy(&bytes).into_owned(),
    }
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
        "a puzzle the API does not know is named"
    );
    seen.lock().unwrap().clear();

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
async fn malformed_links_get_a_page_not_the_extractors_words() {
    let (app, seen) = app().await;
    for (uri, expected) in [
        ("/sheet", "A sheet holds 1 to 12 puzzles."),
        (
            "/sheet/new?count=abc",
            "<code>count</code> must be a number",
        ),
        ("/sheet/new?count=13", "got <code>13</code>"),
        ("/sheet?ids=00008&ids=000Zo", "The link could not be read."),
    ] {
        let response = request(&app, uri, None).await;
        assert_eq!(response.status, StatusCode::BAD_REQUEST, "{uri}");
        assert!(
            response.content_type.starts_with("text/html"),
            "{uri} answered {}",
            response.content_type
        );
        assert!(response.body.contains(expected), "{uri}: {}", response.body);
        assert!(response.body.contains("Back to the form"), "{uri}");
    }
    assert!(seen.lock().unwrap().is_empty());
}

#[tokio::test]
async fn refusals_read_in_the_sheets_language() {
    let (app, _) = app().await;
    let (status, html, _) = get_page(&app, "/sheet?ids=nope1&lang=es").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        html.contains("No hay ningún ejercicio <code>nope1</code>."),
        "{html}"
    );
    assert!(html.contains("Volver al formulario"));
}

#[tokio::test]
async fn an_unknown_language_is_refused_in_the_browsers() {
    let (app, _) = app().await;
    let response = request(&app, "/sheet/new?lang=fr", Some("es-CL")).await;
    assert_eq!(response.status, StatusCode::BAD_REQUEST);
    assert!(response.body.contains("<code>lang</code> debe ser es o en"));

    // The front door stays lenient.
    let response = request(&app, "/?lang=fr", Some("es-CL")).await;
    assert_eq!(response.status, StatusCode::OK);
}

#[tokio::test]
async fn a_blank_title_prints_the_default() {
    let (app, _) = app().await;
    let (status, html, _) = get_page(&app, "/sheet?ids=00008&title=%20%20&lang=en").await;
    assert_eq!(status, StatusCode::OK);
    assert!(html.contains("<h1>Chess exercises</h1>"));
    assert!(!html.contains("<h1></h1>"));
}

#[tokio::test]
async fn a_busy_api_says_so_and_when_to_retry() {
    let (app, _) = app().await;
    let response = request(&app, &format!("/sheet?ids={BUSY_ID}&lang=en"), None).await;
    assert_eq!(response.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(
        response.retry_after.as_deref(),
        Some(BUSY_RETRY_AFTER.to_string().as_str())
    );
    assert!(
        response
            .body
            .contains(&format!("busy. Try again in {BUSY_RETRY_AFTER} seconds")),
        "{}",
        response.body
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

#[tokio::test]
async fn a_sheet_costs_the_api_only_what_it_has_not_seen() {
    let (app, seen) = app().await;
    let (_, _, location) = get_page(&app, "/sheet/new?count=2&lang=en").await;
    let location = location.expect("redirect");
    seen.lock().unwrap().clear();

    get_page(&app, &location).await;
    let first: Vec<String> = seen.lock().unwrap().drain(..).collect();
    assert!(
        first.iter().all(|path| path.ends_with("/solution")),
        "the pick already brought the puzzles: {first:?}"
    );
    assert_eq!(first.len(), 2);

    get_page(&app, &location).await;
    assert!(
        seen.lock().unwrap().is_empty(),
        "a reopened sheet costs nothing"
    );
}

#[tokio::test]
async fn a_sheet_may_be_kept_but_an_error_may_not() {
    let (app, _) = app().await;
    let response = request(&app, "/sheet?ids=00008&lang=en", None).await;
    assert_eq!(
        response.cache_control.as_deref(),
        Some("public, max-age=86400")
    );
    assert!(
        !response.vary.iter().any(|value| value == "accept-language"),
        "the language is in the URL"
    );

    let response = request(&app, "/sheet?ids=00008", Some("es")).await;
    assert!(response.vary.iter().any(|value| value == "accept-language"));

    let response = request(&app, "/sheet?ids=nope1", None).await;
    assert_eq!(response.cache_control, None);
}

#[tokio::test]
async fn every_page_carries_the_security_headers() {
    let (app, _) = app().await;
    for uri in ["/", "/sheet?ids=00008", "/sheet?ids=nope1", "/llms.txt"] {
        let response = app
            .clone()
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();
        let headers = response.headers();
        assert_eq!(
            headers[header::CONTENT_SECURITY_POLICY],
            CONTENT_SECURITY_POLICY,
            "{uri}"
        );
        assert_eq!(headers[header::X_CONTENT_TYPE_OPTIONS], "nosniff", "{uri}");
        assert_eq!(
            headers[header::REFERRER_POLICY],
            "strict-origin-when-cross-origin",
            "{uri}"
        );
    }
}

#[tokio::test]
async fn the_print_button_is_the_one_script_allowed_to_run() {
    let digest = STANDARD.encode(Sha256::digest(PRINT_SCRIPT.as_bytes()));
    assert!(
        CONTENT_SECURITY_POLICY.contains(&format!("'sha256-{digest}'")),
        "the policy must carry the script's hash, sha256-{digest}"
    );

    let (app, _) = app().await;
    let (_, html, _) = get_page(&app, "/sheet?ids=00008").await;
    assert!(html.contains(&format!("<script>{PRINT_SCRIPT}</script>")));
    assert_eq!(html.matches("<script").count(), 1);
    assert!(
        !html.contains("onclick"),
        "inline handlers would be blocked"
    );
}
