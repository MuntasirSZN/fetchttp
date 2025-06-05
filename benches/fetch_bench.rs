//! Criterion benchmarks for fetch operations

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use fetch::*;
use std::hint::black_box;
use std::time::Duration;
use tokio::runtime::Runtime;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn create_runtime() -> Runtime {
    Runtime::new().unwrap()
}

async fn setup_mock_server() -> MockServer {
    let mock_server = MockServer::start().await;

    // GET endpoint
    Mock::given(method("GET"))
        .and(path("/bench"))
        .respond_with(ResponseTemplate::new(200).set_body_string("benchmark response"))
        .mount(&mock_server)
        .await;

    // POST endpoint
    Mock::given(method("POST"))
        .and(path("/bench"))
        .respond_with(ResponseTemplate::new(201).set_body_string("created"))
        .mount(&mock_server)
        .await;

    // JSON endpoint
    Mock::given(method("POST"))
        .and(path("/json"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(r#"{"status": "success"}"#)
                .insert_header("content-type", "application/json"),
        )
        .mount(&mock_server)
        .await;

    // Large response endpoint
    let large_body = "x".repeat(1024 * 100); // 100KB
    Mock::given(method("GET"))
        .and(path("/large"))
        .respond_with(ResponseTemplate::new(200).set_body_string(large_body))
        .mount(&mock_server)
        .await;

    mock_server
}

fn bench_headers_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("headers");

    group.bench_function("create", |b| b.iter(|| black_box(Headers::new())));

    group.bench_function("set_single", |b| {
        b.iter(|| {
            let mut headers = Headers::new();
            headers.set("content-type", "application/json").unwrap();
            black_box(())
        })
    });

    group.bench_function("set_multiple", |b| {
        b.iter(|| {
            let mut headers = Headers::new();
            headers.set("content-type", "application/json").unwrap();
            headers.set("accept", "application/json").unwrap();
            headers.set("user-agent", "fetch-rs/0.1.0").unwrap();
            headers.set("authorization", "Bearer token").unwrap();
            headers.set("x-custom", "value").unwrap();
            black_box(())
        })
    });

    group.bench_function("get", |b| {
        let mut headers = Headers::new();
        headers.set("content-type", "application/json").unwrap();

        b.iter(|| black_box(headers.get("content-type").unwrap()))
    });

    group.bench_function("has", |b| {
        let mut headers = Headers::new();
        headers.set("content-type", "application/json").unwrap();

        b.iter(|| black_box(headers.has("content-type").unwrap()))
    });

    group.bench_function("append", |b| {
        let mut headers = Headers::new();
        headers.set("accept", "application/json").unwrap();

        b.iter(|| {
            let mut h = headers.clone();
            h.append("accept", "text/plain").unwrap();
            black_box(())
        })
    });

    group.finish();
}

fn bench_body_operations(c: &mut Criterion) {
    let rt = create_runtime();
    let mut group = c.benchmark_group("body");

    group.bench_function("create_text", |b| {
        b.iter(|| black_box(ReadableStream::from_text("Hello, World!")))
    });

    group.bench_function("create_bytes", |b| {
        let data = vec![0u8; 1024];
        b.iter(|| black_box(ReadableStream::from_bytes(bytes::Bytes::from(data.clone()))))
    });

    group.bench_function("create_json", |b| {
        let value = serde_json::json!({"key": "value", "number": 42});
        b.iter(|| black_box(ReadableStream::from_json(&value)))
    });

    group.bench_function("text_consumption", |b| {
        b.to_async(&rt).iter(|| async {
            let stream = ReadableStream::from_text("Hello, World!");
            black_box(stream.text().await.unwrap())
        })
    });

    group.bench_function("json_consumption", |b| {
        let value = serde_json::json!({"key": "value", "number": 42});
        b.to_async(&rt).iter(|| async {
            let stream = ReadableStream::from_json(&value);
            let result: serde_json::Value = stream.json().await.unwrap();
            black_box(result)
        })
    });

    group.bench_function("bytes_consumption", |b| {
        let data = vec![0u8; 1024];
        b.to_async(&rt).iter(|| async {
            let stream = ReadableStream::from_bytes(bytes::Bytes::from(data.clone()));
            black_box(stream.array_buffer().await.unwrap())
        })
    });

    group.finish();
}

fn bench_request_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("request");

    group.bench_function("create_simple", |b| {
        b.iter(|| black_box(Request::new("https://example.com", None).unwrap()))
    });

    group.bench_function("create_with_headers", |b| {
        b.iter(|| {
            let mut headers = Headers::new();
            headers.set("content-type", "application/json").unwrap();
            headers.set("accept", "application/json").unwrap();

            let mut init = RequestInit::new();
            init.headers = Some(headers);

            black_box(Request::new("https://example.com", Some(init)).unwrap())
        })
    });

    group.bench_function("create_with_body", |b| {
        b.iter(|| {
            let mut init = RequestInit::new();
            init.method = Some("POST".to_string());
            init.body = Some(ReadableStream::from_text("test body"));

            black_box(Request::new("https://example.com", Some(init)).unwrap())
        })
    });

    group.bench_function("create_full_init", |b| {
        b.iter(|| {
            let mut headers = Headers::new();
            headers.set("content-type", "application/json").unwrap();
            headers.set("accept", "application/json").unwrap();
            headers.set("user-agent", "fetch-rs/0.1.0").unwrap();

            let mut init = RequestInit::new();
            init.method = Some("POST".to_string());
            init.headers = Some(headers);
            init.body = Some(ReadableStream::from_text("test body"));
            init.mode = Some(RequestMode::Cors);
            init.credentials = Some(RequestCredentials::Include);

            black_box(Request::new("https://example.com", Some(init)).unwrap())
        })
    });

    group.finish();
}

fn bench_response_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("response");

    group.bench_function("create_simple", |b| {
        b.iter(|| black_box(Response::new(None, None).unwrap()))
    });

    group.bench_function("create_with_headers", |b| {
        b.iter(|| {
            let mut headers = Headers::new();
            headers.set("content-type", "application/json").unwrap();
            headers.set("x-custom", "value").unwrap();

            let mut init = ResponseInit::new();
            init.headers = Some(headers);

            black_box(Response::new(None, Some(init)).unwrap())
        })
    });

    group.bench_function("create_with_body", |b| {
        b.iter(|| {
            let body = ReadableStream::from_text("response body");
            black_box(Response::new(Some(body), None).unwrap())
        })
    });

    group.bench_function("error", |b| b.iter(|| black_box(Response::error())));

    group.bench_function("redirect", |b| {
        b.iter(|| black_box(Response::redirect("https://example.com", Some(302)).unwrap()))
    });

    group.finish();
}

fn bench_fetch_operations(c: &mut Criterion) {
    let rt = create_runtime();
    let mock_server = rt.block_on(setup_mock_server());
    let base_url = mock_server.uri();

    let mut group = c.benchmark_group("fetch");
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(100);

    group.bench_function("get_request", |b| {
        let url = format!("{}/bench", base_url);
        b.to_async(&rt).iter(|| async {
            let response = fetch(&url, None).await.unwrap();
            black_box(response.text().await.unwrap())
        })
    });

    group.bench_function("post_request", |b| {
        let url = format!("{}/bench", base_url);
        b.to_async(&rt).iter(|| async {
            let mut init = RequestInit::new();
            init.method = Some("POST".to_string());
            init.body = Some(ReadableStream::from_text("test data"));

            let response = fetch(&url, Some(init)).await.unwrap();
            black_box(response.text().await.unwrap())
        })
    });

    group.bench_function("json_request", |b| {
        let url = format!("{}/json", base_url);
        b.to_async(&rt).iter(|| async {
            let data = serde_json::json!({"key": "value"});

            let mut init = RequestInit::new();
            init.method = Some("POST".to_string());
            init.body = Some(ReadableStream::from_json(&data));

            let response = fetch(&url, Some(init)).await.unwrap();
            let result: serde_json::Value = response.json().await.unwrap();
            black_box(result)
        })
    });

    group.bench_function("large_response", |b| {
        let url = format!("{}/large", base_url);
        b.to_async(&rt).iter(|| async {
            let response = fetch(&url, None).await.unwrap();
            black_box(response.text().await.unwrap())
        })
    });

    for concurrent_requests in [1, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_requests", concurrent_requests),
            concurrent_requests,
            |b, &concurrent_requests| {
                let url = format!("{}/bench", base_url);
                b.to_async(&rt).iter(|| async {
                    let futures: Vec<_> = (0..concurrent_requests)
                        .map(|_| {
                            let url_clone = url.clone();
                            async move { fetch(&url_clone, None).await }
                        })
                        .collect();

                    let responses = futures::future::join_all(futures).await;
                    for response in responses {
                        black_box(response.unwrap().text().await.unwrap());
                    }
                })
            },
        );
    }

    group.finish();
}

fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory");

    group.bench_function("headers_memory_100", |b| {
        b.iter(|| {
            let mut headers_vec = Vec::new();
            for i in 0..100 {
                let mut headers = Headers::new();
                headers
                    .set(&format!("header-{}", i), &format!("value-{}", i))
                    .unwrap();
                headers_vec.push(headers);
            }
            black_box(headers_vec)
        })
    });

    group.bench_function("requests_memory_100", |b| {
        b.iter(|| {
            let mut requests_vec = Vec::new();
            for i in 0..100 {
                let request = Request::new(&format!("https://example.com/{}", i), None).unwrap();
                requests_vec.push(request);
            }
            black_box(requests_vec)
        })
    });

    group.bench_function("responses_memory_100", |b| {
        b.iter(|| {
            let mut responses_vec = Vec::new();
            for i in 0..100 {
                let response = Response::new(
                    Some(ReadableStream::from_text(&format!("body-{}", i))),
                    None,
                )
                .unwrap();
                responses_vec.push(response);
            }
            black_box(responses_vec)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_headers_operations,
    bench_body_operations,
    bench_request_creation,
    bench_response_creation,
    bench_fetch_operations,
    bench_memory_usage
);

criterion_main!(benches);
