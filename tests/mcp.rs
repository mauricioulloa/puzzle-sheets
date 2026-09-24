//! The MCP surface, exercised over the real HTTP transport against a
//! stand-in for chess-puzzle-api.

mod common;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{BUSY_RETRY_AFTER, BUSY_THEME, PUBLIC_URL, app, app_against};
use http_body_util::BodyExt;
use puzzle_sheets::chess::options::{LEVELS, THEMES};
use serde_json::{Value, json};
use tower::ServiceExt;

async fn rpc(app: &Router, body: Value) -> Value {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mcp")
                .header("content-type", "application/json")
                .header("accept", "application/json, text/event-stream")
                // rmcp rejects a request without Host as DNS-rebinding defence.
                // Real clients always send it; a hand-built test request does not.
                .header("host", "localhost")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .expect("request");

    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    assert_eq!(
        status,
        StatusCode::OK,
        "MCP transport failed: {}",
        String::from_utf8_lossy(&bytes)
    );
    serde_json::from_slice(&bytes).expect("json-rpc response")
}

async fn call(app: &Router, tool: &str, args: Value) -> Value {
    rpc(
        app,
        json!({
            "jsonrpc": "2.0", "id": 1, "method": "tools/call",
            "params": {"name": tool, "arguments": args}
        }),
    )
    .await
}

fn structured(response: &Value) -> &Value {
    &response["result"]["structuredContent"]
}

#[tokio::test]
async fn initialize_describes_the_server_and_how_to_use_it() {
    let (app, _) = app().await;
    let response = rpc(
        &app,
        json!({
            "jsonrpc": "2.0", "id": 1, "method": "initialize",
            "params": {"protocolVersion": "2025-11-25", "capabilities": {},
                       "clientInfo": {"name": "test", "version": "1"}}
        }),
    )
    .await;

    let result = &response["result"];
    assert_eq!(result["serverInfo"]["name"], "puzzle-sheets");
    assert!(!result["capabilities"]["tools"].is_null());

    // The instructions carry the things a model gets wrong unprompted.
    let instructions = result["instructions"].as_str().expect("instructions");
    assert!(instructions.contains("list_options"));
    assert!(instructions.contains("worksheet_url"));
    assert!(instructions.contains("answers=none"));
}

#[tokio::test]
async fn every_tool_is_advertised_with_a_schema() {
    let (app, _) = app().await;
    let response = rpc(
        &app,
        json!({"jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {}}),
    )
    .await;

    let tools = response["result"]["tools"].as_array().expect("tools");
    let names: Vec<&str> = tools
        .iter()
        .map(|tool| tool["name"].as_str().unwrap())
        .collect();

    for expected in ["list_options", "create_worksheet"] {
        assert!(names.contains(&expected), "{expected} is not advertised");
    }

    for tool in tools {
        assert!(
            tool["description"].as_str().unwrap().len() > 40,
            "{} needs a description a model can act on",
            tool["name"]
        );
        assert!(!tool["inputSchema"].is_null());
    }
}

#[tokio::test]
async fn agents_can_list_options_and_make_a_sheet() {
    let (app, _) = app().await;

    let options = call(&app, "list_options", json!({"lang": "en"})).await;
    let options = structured(&options);
    assert_eq!(
        options["levels"].as_array().expect("levels").len(),
        LEVELS.len()
    );
    assert_eq!(
        options["themes"].as_array().expect("themes").len(),
        THEMES.len()
    );

    let created = call(
        &app,
        "create_worksheet",
        json!({"level": "beginner", "theme": "fork", "count": 2, "answers": "none"}),
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
    let response = call(&app, "create_worksheet", json!({"theme": "chessboxing"})).await;
    let message = response["error"]["message"].as_str().expect("an error");
    assert!(
        message.contains("zugzwang"),
        "it lists the themes that do exist"
    );

    let response = call(&app, "create_worksheet", json!({"answers": "footer"})).await;
    let message = response["error"]["message"].as_str().expect("an error");
    assert_eq!(message, "`answers` must be page or none; got `footer`.");
}

#[tokio::test]
async fn a_busy_api_tells_the_agent_when_to_retry() {
    let (app, _) = app().await;
    let response = call(&app, "create_worksheet", json!({"theme": BUSY_THEME})).await;
    let message = response["error"]["message"].as_str().expect("an error");
    assert_eq!(
        message,
        format!("The puzzle service is busy. Try again in {BUSY_RETRY_AFTER}s.")
    );
}

#[tokio::test]
async fn an_unreachable_api_is_an_error_not_an_empty_sheet() {
    // Nothing listens on port 9 (discard) on a test machine.
    let app = app_against("http://127.0.0.1:9");
    let response = call(&app, "create_worksheet", json!({})).await;
    let message = response["error"]["message"].as_str().expect("an error");
    assert!(message.contains("puzzle service"), "{message}");
}
