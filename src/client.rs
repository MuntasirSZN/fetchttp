//! HTTP client implementation using hyper.
//!
//! This module provides the core [`fetch`] function that implements the WHATWG Fetch API
//! specification. It uses hyper as the underlying HTTP client with TLS support.

use crate::error::{AbortError, FetchError, NetworkError, Result};
use crate::{Headers, ReadableStream, Request, RequestInit, Response};
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use std::sync::OnceLock;

/// Global HTTP client instance.
///
/// This client is shared across all fetch operations to enable connection pooling
/// and improve performance. It's initialized lazily on first use.
static CLIENT: OnceLock<
    Client<
        hyper_tls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>,
        http_body_util::Full<bytes::Bytes>,
    >,
> = OnceLock::new();

/// Get or initialize the global HTTP client.
///
/// The client is configured with HTTPS support and uses the Tokio executor.
/// Connection pooling is handled automatically by hyper.
fn get_client() -> &'static Client<
    hyper_tls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>,
    http_body_util::Full<bytes::Bytes>,
> {
    CLIENT.get_or_init(|| {
        let https = hyper_tls::HttpsConnector::new();
        Client::builder(TokioExecutor::new()).build(https)
    })
}

/// Perform an HTTP request using the Fetch API.
///
/// This function implements the WHATWG Fetch specification for making HTTP requests.
/// It supports all standard HTTP methods, custom headers, request bodies, and
/// abort signals.
///
/// # Arguments
///
/// * `input` - The URL to fetch
/// * `init` - Optional request configuration
///
/// # Returns
///
/// A [`Response`] object on success, or a [`FetchError`] on failure.
///
/// # Examples
///
/// ```rust
/// use fetchttp::*;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Simple GET request
///     let response = fetch("https://httpbin.org/get", None).await?;
///     println!("Status: {}", response.status());
///     
///     // POST request with JSON body
///     let mut init = RequestInit::new();
///     init.method = Some("POST".to_string());
///     init.body = Some(ReadableStream::from_json(&serde_json::json!({
///         "key": "value"
///     })));
///     
///     let response = fetch("https://httpbin.org/post", Some(init)).await?;
///     let result: serde_json::Value = response.json().await?;
///     println!("Response: {}", result);
///     
///     Ok(())
/// }
/// ```
///
/// # Errors
///
/// This function can return the following errors:
///
/// * [`AbortError`] - If the request was aborted via an abort signal
/// * [`NetworkError`] - For network-related failures (DNS, connection, etc.)
/// * [`TypeError`] - For invalid URLs, methods, or other type-related errors
pub async fn fetch(input: &str, init: Option<RequestInit>) -> Result<Response> {
    // Create the request object, which validates URL and options
    let mut request = Request::new(input, init)?;

    // Check if the request was aborted before sending
    if let Some(signal) = request.signal() {
        if signal.aborted() {
            return Err(FetchError::Abort(AbortError::new(
                "The operation was aborted",
            )));
        }
    }

    let client = get_client();

    // Convert the method string to hyper's Method type
    let method = http::Method::from_bytes(request.method().as_bytes())
        .map_err(|_| FetchError::Network(NetworkError::new("Invalid method")))?;

    // Start building the HTTP request
    let mut http_request = http::Request::builder()
        .method(method)
        .uri(request.get_url().as_str());

    // Add headers to the request
    let header_map = request.headers().to_http_headers()?;
    for (name, value) in header_map {
        if let Some(header_name) = name {
            http_request = http_request.header(header_name, value);
        }
    }

    // Add the body if present
    let body = match request.take_body() {
        Some(body) => {
            let bytes = body.to_bytes().await?;
            http_body_util::Full::new(bytes)
        }
        None => http_body_util::Full::new(bytes::Bytes::new()),
    };

    // Finalize the request
    let http_request = http_request.body(body)?;

    // Send the request
    let http_response = client.request(http_request).await?;

    // Process the response
    let (parts, incoming) = http_response.into_parts();
    let headers = Headers::from_http_headers(&parts.headers);
    let status_text = parts.status.canonical_reason().unwrap_or("").to_string();

    // Create the response object
    let mut response = Response::from_parts(
        parts.status.as_u16(),
        status_text,
        headers,
        request.get_url().to_string(),
        false, // redirected flag - would need redirect handling for true implementation
    );

    // Read the response body
    let body_bytes = http_body_util::BodyExt::collect(incoming)
        .await
        .map_err(|e| FetchError::Network(NetworkError::new(&e.to_string())))?
        .to_bytes();

    // Set the body if it's not empty
    if !body_bytes.is_empty() {
        response.set_body(ReadableStream::from_bytes(body_bytes));
    }

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_initialization() {
        let _client = get_client();
        // Client should be initialized without panicking
    }

    #[tokio::test]
    async fn test_fetch_invalid_url() {
        let result = fetch("not-a-url", None).await;
        assert!(result.is_err());
    }
}
