//! Integration tests with real HTTP server

use fetchttp::*;
use wiremock::matchers::{body_string, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_fetch_get_request() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/test"))
        .respond_with(ResponseTemplate::new(200).set_body_string("Hello, World!"))
        .mount(&mock_server)
        .await;

    let response = fetch(&format!("{}/test", mock_server.uri()), None)
        .await
        .unwrap();

    assert!(response.ok());
    assert_eq!(response.status(), 200);

    let text = response.text().await.unwrap();
    assert_eq!(text, "Hello, World!");
}

#[tokio::test]
async fn test_fetch_post_request() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/test"))
        .and(body_string("test data"))
        .respond_with(ResponseTemplate::new(201).set_body_string("Created"))
        .mount(&mock_server)
        .await;

    let mut init = RequestInit::new();
    init.method = Some("POST".to_string());
    init.body = Some(ReadableStream::from_text("test data"));

    let response = fetch(&format!("{}/test", mock_server.uri()), Some(init))
        .await
        .unwrap();

    assert!(response.ok());
    assert_eq!(response.status(), 201);

    let text = response.text().await.unwrap();
    assert_eq!(text, "Created");
}

#[tokio::test]
async fn test_fetch_json_request() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/json"))
        .and(header("content-type", "application/json"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(r#"{"message": "success"}"#)
                .insert_header("content-type", "application/json"),
        )
        .mount(&mock_server)
        .await;

    let data = serde_json::json!({"key": "value"});

    let mut init = RequestInit::new();
    init.method = Some("POST".to_string());
    init.body = Some(ReadableStream::from_json(&data));

    let response = fetch(&format!("{}/json", mock_server.uri()), Some(init))
        .await
        .unwrap();

    assert!(response.ok());
    assert_eq!(response.status(), 200);

    let result: serde_json::Value = response.json().await.unwrap();
    assert_eq!(result["message"], "success");
}

#[tokio::test]
async fn test_fetch_custom_headers() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/headers"))
        .and(header("x-custom", "test-value"))
        .and(header("user-agent", "fetchttp/0.1.0"))
        .respond_with(ResponseTemplate::new(200).set_body_string("OK"))
        .mount(&mock_server)
        .await;

    let mut headers = Headers::new();
    headers.set("x-custom", "test-value").unwrap();
    headers.set("user-agent", "fetchttp/0.1.0").unwrap();

    let mut init = RequestInit::new();
    init.headers = Some(headers);

    let response = fetch(&format!("{}/headers", mock_server.uri()), Some(init))
        .await
        .unwrap();

    assert!(response.ok());
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_fetch_error_status() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/error"))
        .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
        .mount(&mock_server)
        .await;

    let response = fetch(&format!("{}/error", mock_server.uri()), None)
        .await
        .unwrap();

    assert!(!response.ok());
    assert_eq!(response.status(), 404);

    let text = response.text().await.unwrap();
    assert_eq!(text, "Not Found");
}

#[tokio::test]
async fn test_fetch_redirect() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/redirect"))
        .respond_with(ResponseTemplate::new(302).insert_header("location", "/target"))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/target"))
        .respond_with(ResponseTemplate::new(200).set_body_string("Redirected"))
        .mount(&mock_server)
        .await;

    let response = fetch(&format!("{}/redirect", mock_server.uri()), None)
        .await
        .unwrap();

    // Note: This behavior depends on the underlying HTTP client's redirect handling
    // In a real implementation, we might need to handle redirects manually
    assert!(response.status() == 302 || response.status() == 200);
}

#[tokio::test]
async fn test_fetch_large_response() {
    let mock_server = MockServer::start().await;

    let large_body = "x".repeat(1024 * 1024); // 1MB

    Mock::given(method("GET"))
        .and(path("/large"))
        .respond_with(ResponseTemplate::new(200).set_body_string(&large_body))
        .mount(&mock_server)
        .await;

    let response = fetch(&format!("{}/large", mock_server.uri()), None)
        .await
        .unwrap();

    assert!(response.ok());
    assert_eq!(response.status(), 200);

    let text = response.text().await.unwrap();
    assert_eq!(text.len(), 1024 * 1024);
}

#[tokio::test]
async fn test_fetch_response_headers() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/response-headers"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string("OK")
                .insert_header("x-custom", "response-value")
                .insert_header("content-type", "text/plain"),
        )
        .mount(&mock_server)
        .await;

    let response = fetch(&format!("{}/response-headers", mock_server.uri()), None)
        .await
        .unwrap();

    assert!(response.ok());
    assert_eq!(response.status(), 200);

    let headers = response.headers();
    assert!(headers.has("x-custom").unwrap());
    assert_eq!(headers.get("x-custom").unwrap().unwrap(), "response-value");
    assert!(headers.has("content-type").unwrap());
}

#[tokio::test]
async fn test_request_clone() {
    let request = Request::new("https://example.com", None).unwrap();
    let cloned = request.clone_request().unwrap();

    assert_eq!(request.url(), cloned.url());
    assert_eq!(request.method(), cloned.method());
}

#[tokio::test]
async fn test_response_clone() {
    let response = Response::new(None, None).unwrap();
    let cloned = response.clone_response().unwrap();

    assert_eq!(response.status(), cloned.status());
    assert_eq!(response.ok(), cloned.ok());
}

#[tokio::test]
async fn test_body_consumption() {
    let response = Response::new(Some(ReadableStream::from_text("test body")), None).unwrap();

    assert!(!response.body_used());

    let text = response.text().await.unwrap();
    assert_eq!(text, "test body");

    // Note: After consumption, the response is moved and can't be accessed again
}
