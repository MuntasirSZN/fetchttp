//! IAI-Callgrind benchmarks for instruction counting and cache analysis

use fetch::*;
use iai_callgrind::{
    library_benchmark, library_benchmark_group, main, EventKind, FlamegraphConfig,
    LibraryBenchmarkConfig, RegressionConfig, Tool, ValgrindTool,
};

#[library_benchmark]
fn headers_create() -> Headers {
    Headers::new()
}

#[library_benchmark]
fn headers_set_single() -> Headers {
    let mut headers = Headers::new();
    headers.set("content-type", "application/json").unwrap();
    headers
}

#[library_benchmark]
fn headers_set_multiple() -> Headers {
    let mut headers = Headers::new();
    headers.set("content-type", "application/json").unwrap();
    headers.set("accept", "application/json").unwrap();
    headers.set("user-agent", "fetch-rs/0.1.0").unwrap();
    headers.set("authorization", "Bearer token").unwrap();
    headers.set("x-custom", "value").unwrap();
    headers
}

#[library_benchmark]
fn headers_get() -> Option<String> {
    let mut headers = Headers::new();
    headers.set("content-type", "application/json").unwrap();
    headers.get("content-type").unwrap()
}

#[library_benchmark]
fn headers_has() -> bool {
    let mut headers = Headers::new();
    headers.set("content-type", "application/json").unwrap();
    headers.has("content-type").unwrap()
}

#[library_benchmark]
fn headers_append() -> Headers {
    let mut headers = Headers::new();
    headers.set("accept", "application/json").unwrap();
    headers.append("accept", "text/plain").unwrap();
    headers
}

#[library_benchmark]
fn headers_validation_success() -> Headers {
    let mut headers = Headers::new();
    headers.set("x-custom-header", "valid-value").unwrap();
    headers
}

#[library_benchmark]
fn headers_validation_failure() {
    let mut headers = Headers::new();
    let _ = headers.set("", "value");
}

#[library_benchmark]
fn body_create_text() -> ReadableStream {
    ReadableStream::from_text("Hello, World!")
}

#[library_benchmark]
fn body_create_bytes() -> ReadableStream {
    let data = vec![0u8; 1024];
    ReadableStream::from_bytes(bytes::Bytes::from(data))
}

#[library_benchmark]
fn body_create_json() -> ReadableStream {
    let value = serde_json::json!({"key": "value", "number": 42});
    ReadableStream::from_json(&value)
}

#[library_benchmark]
fn body_create_large_text() -> ReadableStream {
    let large_text = "x".repeat(10240); // 10KB
    ReadableStream::from_text(&large_text)
}

#[library_benchmark]
fn body_create_large_bytes() -> ReadableStream {
    let data = vec![0u8; 10240]; // 10KB
    ReadableStream::from_bytes(bytes::Bytes::from(data))
}

#[library_benchmark]
fn request_create_simple() -> Request {
    Request::new("https://example.com", None).unwrap()
}

#[library_benchmark]
fn request_create_with_headers() -> Request {
    let mut headers = Headers::new();
    headers.set("content-type", "application/json").unwrap();
    headers.set("accept", "application/json").unwrap();

    let mut init = RequestInit::new();
    init.headers = Some(headers);

    Request::new("https://example.com", Some(init)).unwrap()
}

#[library_benchmark]
fn request_create_with_body() -> Request {
    let mut init = RequestInit::new();
    init.method = Some("POST".to_string());
    init.body = Some(ReadableStream::from_text("test body"));

    Request::new("https://example.com", Some(init)).unwrap()
}

#[library_benchmark]
fn request_create_full_init() -> Request {
    let mut headers = Headers::new();
    headers.set("content-type", "application/json").unwrap();
    headers.set("accept", "application/json").unwrap();
    headers.set("user-agent", "fetch-rs/0.1.0").unwrap();
    headers.set("authorization", "Bearer token").unwrap();

    let mut init = RequestInit::new();
    init.method = Some("POST".to_string());
    init.headers = Some(headers);
    init.body = Some(ReadableStream::from_text("test body"));
    init.mode = Some(RequestMode::Cors);
    init.credentials = Some(RequestCredentials::Include);
    init.cache = Some(RequestCache::NoCache);
    init.redirect = Some(RequestRedirect::Follow);

    Request::new("https://example.com/api/v1/users", Some(init)).unwrap()
}

#[library_benchmark]
fn request_method_validation_success() -> Request {
    let mut init = RequestInit::new();
    init.method = Some("POST".to_string());
    Request::new("https://example.com", Some(init)).unwrap()
}

#[library_benchmark]
fn request_method_validation_failure() {
    let mut init = RequestInit::new();
    init.method = Some("".to_string());
    let _ = Request::new("https://example.com", Some(init));
}

#[library_benchmark]
fn request_url_validation_success() -> Request {
    Request::new("https://example.com/path?query=value#fragment", None).unwrap()
}

#[library_benchmark]
fn request_url_validation_failure() {
    let _ = Request::new("not-a-valid-url", None);
}

#[library_benchmark]
fn response_create_simple() -> Response {
    Response::new(None, None).unwrap()
}

#[library_benchmark]
fn response_create_with_headers() -> Response {
    let mut headers = Headers::new();
    headers.set("content-type", "application/json").unwrap();
    headers.set("x-custom", "value").unwrap();

    let mut init = ResponseInit::new();
    init.headers = Some(headers);

    Response::new(None, Some(init)).unwrap()
}

#[library_benchmark]
fn response_create_with_body() -> Response {
    let body = ReadableStream::from_text("response body");
    Response::new(Some(body), None).unwrap()
}

#[library_benchmark]
fn response_create_full() -> Response {
    let mut headers = Headers::new();
    headers.set("content-type", "application/json").unwrap();
    headers.set("cache-control", "no-cache").unwrap();
    headers.set("x-custom", "value").unwrap();

    let mut init = ResponseInit::new();
    init.status = Some(201);
    init.status_text = Some("Created".to_string());
    init.headers = Some(headers);

    let body = ReadableStream::from_json(&serde_json::json!({"status": "created"}));
    Response::new(Some(body), Some(init)).unwrap()
}

#[library_benchmark]
fn response_error() -> Response {
    Response::error()
}

#[library_benchmark]
fn response_redirect() -> Response {
    Response::redirect("https://example.com/new-location", Some(302)).unwrap()
}

#[library_benchmark]
fn response_status_validation_success() -> Response {
    let mut init = ResponseInit::new();
    init.status = Some(404);
    Response::new(None, Some(init)).unwrap()
}

#[library_benchmark]
fn response_status_validation_failure() {
    let mut init = ResponseInit::new();
    init.status = Some(600);
    let _ = Response::new(None, Some(init));
}

#[library_benchmark]
fn abort_signal_create() -> AbortSignal {
    AbortSignal::new()
}

#[library_benchmark]
fn abort_signal_abort() -> AbortSignal {
    AbortSignal::abort(Some("Test abort".to_string()))
}

#[library_benchmark]
fn abort_controller_create() -> AbortController {
    AbortController::new()
}

#[library_benchmark]
fn abort_controller_abort() -> AbortController {
    let controller = AbortController::new();
    controller.abort();
    controller
}

#[library_benchmark]
fn json_serialization_small() -> ReadableStream {
    let value = serde_json::json!({"key": "value"});
    ReadableStream::from_json(&value)
}

#[library_benchmark]
fn json_serialization_medium() -> ReadableStream {
    let value = serde_json::json!({
        "users": [
            {"id": 1, "name": "Alice", "email": "alice@example.com"},
            {"id": 2, "name": "Bob", "email": "bob@example.com"},
            {"id": 3, "name": "Charlie", "email": "charlie@example.com"}
        ],
        "metadata": {
            "total": 3,
            "page": 1,
            "per_page": 10
        }
    });
    ReadableStream::from_json(&value)
}

#[library_benchmark]
fn json_serialization_large() -> ReadableStream {
    let mut users = Vec::new();
    for i in 0..1000 {
        users.push(serde_json::json!({
            "id": i,
            "name": format!("User {}", i),
            "email": format!("user{}@example.com", i),
            "created_at": "2025-06-04T09:59:14Z",
            "updated_at": "2025-06-04T09:59:14Z"
        }));
    }

    let value = serde_json::json!({
        "users": users,
        "metadata": {
            "total": 1000,
            "page": 1,
            "per_page": 1000
        }
    });
    ReadableStream::from_json(&value)
}

#[library_benchmark]
fn clone_operations() -> (Request, Response) {
    let request = Request::new("https://example.com", None).unwrap();
    let response = Response::new(None, None).unwrap();

    let cloned_request = request.clone_request().unwrap();
    let cloned_response = response.clone_response().unwrap();

    (cloned_request, cloned_response)
}

library_benchmark_group!(
    name = headers_bench;
    benchmarks =
        headers_create,
        headers_set_single,
        headers_set_multiple,
        headers_get,
        headers_has,
        headers_append,
        headers_validation_success,
        headers_validation_failure
);

library_benchmark_group!(
    name = body_bench;
    benchmarks =
        body_create_text,
        body_create_bytes,
        body_create_json,
        body_create_large_text,
        body_create_large_bytes
);

library_benchmark_group!(
    name = request_bench;
    benchmarks =
        request_create_simple,
        request_create_with_headers,
        request_create_with_body,
        request_create_full_init,
        request_method_validation_success,
        request_method_validation_failure,
        request_url_validation_success,
        request_url_validation_failure
);

library_benchmark_group!(
    name = response_bench;
    benchmarks =
        response_create_simple,
        response_create_with_headers,
        response_create_with_body,
        response_create_full,
        response_error,
        response_redirect,
        response_status_validation_success,
        response_status_validation_failure
);

library_benchmark_group!(
    name = abort_bench;
    benchmarks =
        abort_signal_create,
        abort_signal_abort,
        abort_controller_create,
        abort_controller_abort
);

library_benchmark_group!(
    name = serialization_bench;
    benchmarks =
        json_serialization_small,
        json_serialization_medium,
        json_serialization_large
);

library_benchmark_group!(
    name = misc_bench;
    benchmarks =
        clone_operations
);

main!(
    config = LibraryBenchmarkConfig::default()
                .tool(Tool::new(ValgrindTool::DHAT))
                .tool(Tool::new(ValgrindTool::Massif))
                .tool(Tool::new(ValgrindTool::BBV))
                .tool(Tool::new(ValgrindTool::Memcheck))
                .tool(Tool::new(ValgrindTool::Helgrind))
                .tool(Tool::new(ValgrindTool::DRD))
                .flamegraph(FlamegraphConfig::default())
                .regression(
                    RegressionConfig::default()
                    .limits([(EventKind::Ir, 5.0)])
                    );
    library_benchmark_groups =
        headers_bench,
        body_bench,
        request_bench,
        response_bench,
        abort_bench,
        serialization_bench,
        misc_bench
);
