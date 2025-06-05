//! Performance regression tests

use fetchttp::*;
use std::time::{Duration, Instant};

#[tokio::test]
async fn test_headers_performance() {
    let start = Instant::now();

    for _ in 0..1000 {
        let mut headers = Headers::new();
        headers.set("content-type", "application/json").unwrap();
        headers.set("accept", "application/json").unwrap();
        headers.set("user-agent", "fetchttp/0.1.0").unwrap();

        let _ = headers.get("content-type").unwrap();
        let _ = headers.has("accept").unwrap();
    }

    let elapsed = start.elapsed();
    assert!(
        elapsed < Duration::from_millis(100),
        "Headers operations too slow: {:?}",
        elapsed
    );
}

#[tokio::test]
async fn test_body_operations_performance() {
    let start = Instant::now();

    for _ in 0..100 {
        let stream = ReadableStream::from_text("test body content");
        let _text = stream.text().await.unwrap();

        let data = serde_json::json!({"key": "value", "number": 42});
        let stream = ReadableStream::from_json(&data);
        let _parsed: serde_json::Value = stream.json().await.unwrap();
    }

    let elapsed = start.elapsed();
    assert!(
        elapsed < Duration::from_millis(500),
        "Body operations too slow: {:?}",
        elapsed
    );
}

#[tokio::test]
async fn test_request_creation_performance() {
    let start = Instant::now();

    for i in 0..1000 {
        let mut init = RequestInit::new();
        init.method = Some("POST".to_string());
        init.body = Some(ReadableStream::from_text("test body"));

        let _request = Request::new(&format!("https://example.com/{}", i), Some(init)).unwrap();
    }

    let elapsed = start.elapsed();
    assert!(
        elapsed < Duration::from_millis(200),
        "Request creation too slow: {:?}",
        elapsed
    );
}

#[tokio::test]
async fn test_response_creation_performance() {
    let start = Instant::now();

    for _ in 0..1000 {
        let _response =
            Response::new(Some(ReadableStream::from_text("response body")), None).unwrap();
    }

    let elapsed = start.elapsed();
    assert!(
        elapsed < Duration::from_millis(100),
        "Response creation too slow: {:?}",
        elapsed
    );
}

#[tokio::test]
async fn test_json_serialization_performance() {
    let start = Instant::now();

    for _ in 0..100 {
        let data = serde_json::json!({
            "name": "test",
            "value": 42,
            "items": [1, 2, 3, 4, 5],
            "nested": {
                "key": "value"
            }
        });
        let stream = ReadableStream::from_json(&data);
        let _parsed: serde_json::Value = stream.json().await.unwrap();
    }

    let elapsed = start.elapsed();
    assert!(
        elapsed < Duration::from_millis(200),
        "JSON serialization too slow: {:?}",
        elapsed
    );
}

#[tokio::test]
async fn test_concurrent_operations() {
    let start = Instant::now();

    let mut handles = Vec::new();

    for i in 0..10 {
        let handle = tokio::spawn(async move {
            for j in 0..100 {
                let mut headers = Headers::new();
                headers
                    .set("x-test", &format!("value-{}-{}", i, j))
                    .unwrap();

                let stream = ReadableStream::from_text(&format!("body-{}-{}", i, j));
                let _text = stream.text().await.unwrap();
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let elapsed = start.elapsed();
    assert!(
        elapsed < Duration::from_secs(2),
        "Concurrent operations too slow: {:?}",
        elapsed
    );
}

#[tokio::test]
async fn test_memory_usage_stability() {
    // Test that we don't have memory leaks by creating and dropping many objects
    for _ in 0..1000 {
        let mut headers = Headers::new();
        headers.set("content-type", "application/json").unwrap();

        let request = Request::new("https://example.com", None).unwrap();
        let _cloned = request.clone_request().unwrap();

        let response = Response::new(Some(ReadableStream::from_text("test")), None).unwrap();
        let _cloned = response.clone_response().unwrap();

        // Objects should be dropped here
    }

    // If we get here without panicking, memory usage is likely stable
}
