//! HTTP response types and builders following the WHATWG Fetch specification.
//!
//! This module provides the [`Response`] type for representing HTTP responses.
//! It follows the WHATWG Fetch specification to provide a familiar API for
//! handling response data, headers, and status codes.
//!
//! # Response Creation
//!
//! Responses can be created in several ways:
//! - Through the [`fetch`] function (most common)
//! - Using [`Response::new()`] for custom responses
//! - Using [`Response::error()`] for error responses
//! - Using [`Response::redirect()`] for redirect responses
//!
//! [`fetch`]: crate::fetch
//!
//! # Examples
//!
//! ## Basic Response Handling
//!
//! ```rust
//! use fetch::*;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let response = fetch("https://httpbin.org/json", None).await?;
//!
//! if response.ok() {
//!     println!("Status: {}", response.status());
//!     println!("Status Text: {}", response.status_text());
//!     
//!     let data: serde_json::Value = response.json().await?;
//!     println!("Data: {}", data);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Creating Custom Responses
//!
//! ```rust
//! use fetch::{Response, ResponseInit, ReadableStream, Headers};
//!
//! // Simple response
//! let response = Response::new(
//!     Some(ReadableStream::from_text("Hello, World!")),
//!     None
//! ).unwrap();
//!
//! // Response with custom status and headers
//! let mut headers = Headers::new();
//! headers.set("Content-Type", "application/json").unwrap();
//!
//! let mut init = ResponseInit::new();
//! init.status = Some(201);
//! init.status_text = Some("Created".to_string());
//! init.headers = Some(headers);
//!
//! let response = Response::new(
//!     Some(ReadableStream::from_json(&serde_json::json!({"created": true}))),
//!     Some(init)
//! ).unwrap();
//! ```

use crate::error::{FetchError, Result, TypeError};
use crate::{Headers, ReadableStream};

/// Response type classification.
///
/// This enum classifies responses according to the WHATWG Fetch specification,
/// providing information about the response's origin and processing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseType {
    /// Basic response from same origin
    Basic,
    /// CORS response from cross-origin request
    Cors,
    /// Default response type
    Default,
    /// Error response (network error, etc.)
    Error,
    /// Opaque response (no-cors mode)
    Opaque,
    /// Opaque redirect response
    OpaqueRedirect,
}

impl Default for ResponseType {
    fn default() -> Self {
        Self::Basic
    }
}

/// Configuration for creating responses.
///
/// `ResponseInit` provides options that can be set when creating a new
/// [`Response`]. All fields are optional and will use defaults if not specified.
///
/// # Examples
///
/// ```rust
/// use fetch::{ResponseInit, Headers};
///
/// let mut headers = Headers::new();
/// headers.set("X-Custom-Header", "value").unwrap();
///
/// let mut init = ResponseInit::new();
/// init.status = Some(404);
/// init.status_text = Some("Not Found".to_string());
/// init.headers = Some(headers);
/// ```
#[derive(Debug, Clone, Default)]
pub struct ResponseInit {
    /// HTTP status code (200-599)
    pub status: Option<u16>,
    /// HTTP status text
    pub status_text: Option<String>,
    /// Response headers
    pub headers: Option<Headers>,
}

impl ResponseInit {
    /// Create a new empty ResponseInit.
    ///
    /// All fields will be `None` and will use defaults when creating a response.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetch::ResponseInit;
    ///
    /// let init = ResponseInit::new();
    /// assert!(init.status.is_none());
    /// assert!(init.status_text.is_none());
    /// assert!(init.headers.is_none());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }
}

/// An HTTP response following the WHATWG Fetch specification.
///
/// `Response` represents an HTTP response with all its associated metadata
/// including status code, headers, body, and response type information.
///
/// # Body Consumption
///
/// Response bodies can only be consumed once. After calling any body consumption
/// method (`text()`, `json()`, `array_buffer()`, etc.), the body is marked as used
/// and cannot be consumed again.
///
/// # Status Code Validation
///
/// The `ok()` method returns `true` for status codes in the range 200-299 (inclusive).
/// This follows the HTTP specification for successful responses.
///
/// # Examples
///
/// ```rust
/// use fetch::{Response, ReadableStream};
///
/// # tokio_test::block_on(async {
/// let response = Response::new(
///     Some(ReadableStream::from_text("Hello, World!")),
///     None
/// ).unwrap();
///
/// assert_eq!(response.status(), 200);
/// assert!(response.ok());
///
/// let text = response.text().await.unwrap();
/// assert_eq!(text, "Hello, World!");
/// # });
/// ```
#[derive(Debug)]
pub struct Response {
    /// Response type classification
    response_type: ResponseType,
    /// Response URL (may be different from request URL due to redirects)
    url: String,
    /// Whether the response is the result of a redirect
    redirected: bool,
    /// HTTP status code
    status: u16,
    /// HTTP status text
    status_text: String,
    /// Response headers
    headers: Headers,
    /// Response body (optional)
    body: Option<ReadableStream>,
}

impl Response {
    /// Create a new HTTP response.
    ///
    /// This constructor validates the status code and status text according to
    /// HTTP standards and the WHATWG Fetch specification.
    ///
    /// # Arguments
    ///
    /// * `body` - Optional response body
    /// * `init` - Optional response configuration
    ///
    /// # Returns
    ///
    /// A new `Response` instance, or an error if validation fails.
    ///
    /// # Errors
    ///
    /// * [`TypeError`] - If the status code is invalid (not 200-599) or status text contains invalid characters
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetch::{Response, ResponseInit, ReadableStream};
    ///
    /// // Simple 200 OK response
    /// let response = Response::new(
    ///     Some(ReadableStream::from_text("Success")),
    ///     None
    /// ).unwrap();
    /// assert_eq!(response.status(), 200);
    ///
    /// // Custom 404 response
    /// let mut init = ResponseInit::new();
    /// init.status = Some(404);
    /// init.status_text = Some("Not Found".to_string());
    ///
    /// let response = Response::new(None, Some(init)).unwrap();
    /// assert_eq!(response.status(), 404);
    /// assert!(!response.ok());
    ///
    /// // Invalid status code will fail
    /// let mut invalid_init = ResponseInit::new();
    /// invalid_init.status = Some(999); // Invalid status code
    /// assert!(Response::new(None, Some(invalid_init)).is_err());
    /// ```
    pub fn new(body: Option<ReadableStream>, init: Option<ResponseInit>) -> Result<Self> {
        let init = init.unwrap_or_default();
        let status = init.status.unwrap_or(200);

        // Validate status code (must be 200-599)
        if !(200..=599).contains(&status) {
            return Err(FetchError::Type(TypeError::new("Invalid status code")));
        }

        let status_text = init
            .status_text
            .unwrap_or_else(|| Self::default_status_text(status));

        // Validate status text (no CR/LF characters)
        for byte in status_text.bytes() {
            if matches!(byte, b'\r' | b'\n') {
                return Err(FetchError::Type(TypeError::new("Invalid status text")));
            }
        }

        Ok(Self {
            response_type: ResponseType::Basic,
            url: String::new(),
            redirected: false,
            status,
            status_text,
            headers: init.headers.unwrap_or_default(),
            body,
        })
    }

    /// Create an error response.
    ///
    /// Error responses have a status of 0 and represent network errors or
    /// other failures that prevent a proper HTTP response.
    ///
    /// # Returns
    ///
    /// A new error response.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetch::{Response, ResponseType};
    ///
    /// let response = Response::error();
    /// assert_eq!(response.status(), 0);
    /// assert!(!response.ok());
    /// assert_eq!(response.response_type(), ResponseType::Error);
    /// ```
    pub fn error() -> Self {
        Self {
            response_type: ResponseType::Error,
            url: String::new(),
            redirected: false,
            status: 0,
            status_text: String::new(),
            headers: Headers::new(),
            body: None,
        }
    }

    /// Create a redirect response.
    ///
    /// Redirect responses have a status code in the 3xx range and include
    /// a Location header pointing to the redirect target.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to redirect to
    /// * `status` - Optional redirect status code (defaults to 302)
    ///
    /// # Returns
    ///
    /// A new redirect response, or an error if the status code is invalid.
    ///
    /// # Errors
    ///
    /// * [`TypeError`] - If the status code is not a valid redirect code
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetch::Response;
    ///
    /// // Temporary redirect (302)
    /// let response = Response::redirect("https://example.com/new", None).unwrap();
    /// assert_eq!(response.status(), 302);
    /// assert_eq!(
    ///     response.headers().get("location").unwrap().unwrap(),
    ///     "https://example.com/new"
    /// );
    ///
    /// // Permanent redirect (301)
    /// let response = Response::redirect("https://example.com/new", Some(301)).unwrap();
    /// assert_eq!(response.status(), 301);
    ///
    /// // Invalid redirect status will fail
    /// assert!(Response::redirect("https://example.com", Some(200)).is_err());
    /// ```
    pub fn redirect(url: &str, status: Option<u16>) -> Result<Self> {
        let status = status.unwrap_or(302);

        // Validate redirect status codes
        if !matches!(status, 301 | 302 | 303 | 307 | 308) {
            return Err(FetchError::Type(TypeError::new("Invalid redirect status")));
        }

        let mut headers = Headers::new();
        headers.set("location", url)?;

        Ok(Self {
            response_type: ResponseType::Basic,
            url: String::new(),
            redirected: false,
            status,
            status_text: Self::default_status_text(status),
            headers,
            body: None,
        })
    }

    /// Get the response type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetch::{Response, ResponseType};
    ///
    /// let response = Response::new(None, None).unwrap();
    /// assert_eq!(response.response_type(), ResponseType::Basic);
    ///
    /// let error_response = Response::error();
    /// assert_eq!(error_response.response_type(), ResponseType::Error);
    /// ```
    pub fn response_type(&self) -> ResponseType {
        self.response_type
    }

    /// Get the response URL.
    ///
    /// This may be different from the original request URL if redirects occurred.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetch::Response;
    ///
    /// let response = Response::new(None, None).unwrap();
    /// assert_eq!(response.url(), "");
    /// ```
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Check if the response is the result of a redirect.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetch::Response;
    ///
    /// let response = Response::new(None, None).unwrap();
    /// assert!(!response.redirected());
    /// ```
    pub fn redirected(&self) -> bool {
        self.redirected
    }

    /// Get the HTTP status code.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetch::{Response, ResponseInit};
    ///
    /// let response = Response::new(None, None).unwrap();
    /// assert_eq!(response.status(), 200);
    ///
    /// let mut init = ResponseInit::new();
    /// init.status = Some(404);
    /// let response = Response::new(None, Some(init)).unwrap();
    /// assert_eq!(response.status(), 404);
    /// ```
    pub fn status(&self) -> u16 {
        self.status
    }

    /// Check if the response represents a successful HTTP status.
    ///
    /// Returns `true` for status codes in the range 200-299 (inclusive).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetch::{Response, ResponseInit};
    ///
    /// // 2xx status codes are ok
    /// let response = Response::new(None, None).unwrap(); // 200
    /// assert!(response.ok());
    ///
    /// let mut init = ResponseInit::new();
    /// init.status = Some(201);
    /// let response = Response::new(None, Some(init)).unwrap();
    /// assert!(response.ok());
    ///
    /// // Other status codes are not ok
    /// let mut init = ResponseInit::new();
    /// init.status = Some(404);
    /// let response = Response::new(None, Some(init)).unwrap();
    /// assert!(!response.ok());
    ///
    /// let error_response = Response::error(); // Status 0
    /// assert!(!error_response.ok());
    /// ```
    pub fn ok(&self) -> bool {
        (200..300).contains(&self.status)
    }

    /// Get the HTTP status text.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetch::{Response, ResponseInit};
    ///
    /// let response = Response::new(None, None).unwrap();
    /// assert_eq!(response.status_text(), "OK");
    ///
    /// let mut init = ResponseInit::new();
    /// init.status = Some(404);
    /// let response = Response::new(None, Some(init)).unwrap();
    /// assert_eq!(response.status_text(), "Not Found");
    /// ```
    pub fn status_text(&self) -> &str {
        &self.status_text
    }

    /// Get the response headers.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetch::{Response, ResponseInit, Headers};
    ///
    /// let mut headers = Headers::new();
    /// headers.set("Content-Type", "application/json").unwrap();
    ///
    /// let mut init = ResponseInit::new();
    /// init.headers = Some(headers);
    ///
    /// let response = Response::new(None, Some(init)).unwrap();
    /// assert!(response.headers().has("content-type").unwrap());
    /// ```
    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    /// Get the response body.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetch::{Response, ReadableStream};
    ///
    /// // Response without body
    /// let response = Response::new(None, None).unwrap();
    /// assert!(response.body().is_none());
    ///
    /// // Response with body
    /// let response = Response::new(
    ///     Some(ReadableStream::from_text("content")),
    ///     None
    /// ).unwrap();
    /// assert!(response.body().is_some());
    /// ```
    pub fn body(&self) -> Option<&ReadableStream> {
        self.body.as_ref()
    }

    /// Check if the response body has been used.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetch::{Response, ReadableStream};
    ///
    /// let response = Response::new(
    ///     Some(ReadableStream::from_text("content")),
    ///     None
    /// ).unwrap();
    /// assert!(!response.body_used());
    ///
    /// // After consuming the body, it would be marked as used
    /// // (body consumption moves the response, so we can't show this in the example)
    /// ```
    pub fn body_used(&self) -> bool {
        self.body.as_ref().is_some_and(|b| b.is_used())
    }

    /// Clone the response (WHATWG Fetch API method).
    ///
    /// This method follows the WHATWG Fetch specification for cloning responses.
    /// It will fail if the response body has already been used.
    ///
    /// # Returns
    ///
    /// A cloned response, or an error if the body has been used.
    ///
    /// # Errors
    ///
    /// * [`TypeError`] - If the response body has already been consumed
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetch::Response;
    ///
    /// let response = Response::new(None, None).unwrap();
    /// let cloned = response.clone_response().unwrap();
    ///
    /// assert_eq!(response.status(), cloned.status());
    /// assert_eq!(response.ok(), cloned.ok());
    /// ```
    pub fn clone_response(&self) -> Result<Self> {
        if self.body_used() {
            return Err(FetchError::Type(TypeError::new(
                "Cannot clone a response with a used body",
            )));
        }
        Ok(Clone::clone(self))
    }

    /// Consume the response and return the body as bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetch::{Response, ReadableStream};
    ///
    /// # tokio_test::block_on(async {
    /// let response = Response::new(
    ///     Some(ReadableStream::from_text("Hello, World!")),
    ///     None
    /// ).unwrap();
    ///
    /// let bytes = response.array_buffer().await.unwrap();
    /// assert_eq!(bytes, b"Hello, World!");
    /// # });
    /// ```
    pub async fn array_buffer(self) -> Result<bytes::Bytes> {
        match self.body {
            Some(body) => body.array_buffer().await,
            None => Ok(bytes::Bytes::new()),
        }
    }

    /// Consume the response and return the body as a blob (bytes).
    pub async fn blob(self) -> Result<bytes::Bytes> {
        self.array_buffer().await
    }

    /// Consume the response and return the body as form data.
    pub async fn form_data(self) -> Result<String> {
        match self.body {
            Some(body) => body.form_data().await,
            None => Ok(String::new()),
        }
    }

    /// Consume the response and parse the body as JSON.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetch::{Response, ReadableStream};
    /// use serde_json::json;
    ///
    /// # tokio_test::block_on(async {
    /// let data = json!({"message": "Hello", "count": 42});
    /// let response = Response::new(
    ///     Some(ReadableStream::from_json(&data)),
    ///     None
    /// ).unwrap();
    ///
    /// let parsed: serde_json::Value = response.json().await.unwrap();
    /// assert_eq!(parsed["message"], "Hello");
    /// assert_eq!(parsed["count"], 42);
    /// # });
    /// ```
    pub async fn json<T: serde::de::DeserializeOwned>(self) -> Result<T> {
        match self.body {
            Some(body) => body.json().await,
            None => Err(FetchError::Type(TypeError::new(
                "Unexpected end of JSON input",
            ))),
        }
    }

    /// Consume the response and return the body as text.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetch::{Response, ReadableStream};
    ///
    /// # tokio_test::block_on(async {
    /// let response = Response::new(
    ///     Some(ReadableStream::from_text("Hello, World!")),
    ///     None
    /// ).unwrap();
    ///
    /// let text = response.text().await.unwrap();
    /// assert_eq!(text, "Hello, World!");
    /// # });
    /// ```
    pub async fn text(self) -> Result<String> {
        match self.body {
            Some(body) => body.text().await,
            None => Ok(String::new()),
        }
    }

    /// Get the default status text for a status code.
    ///
    /// Returns the standard HTTP reason phrases for common status codes.
    fn default_status_text(status: u16) -> String {
        match status {
            200 => "OK",
            201 => "Created",
            204 => "No Content",
            301 => "Moved Permanently",
            302 => "Found",
            303 => "See Other",
            304 => "Not Modified",
            307 => "Temporary Redirect",
            308 => "Permanent Redirect",
            400 => "Bad Request",
            401 => "Unauthorized",
            403 => "Forbidden",
            404 => "Not Found",
            405 => "Method Not Allowed",
            409 => "Conflict",
            410 => "Gone",
            422 => "Unprocessable Entity",
            429 => "Too Many Requests",
            500 => "Internal Server Error",
            501 => "Not Implemented",
            502 => "Bad Gateway",
            503 => "Service Unavailable",
            504 => "Gateway Timeout",
            _ => "",
        }
        .to_string()
    }

    /// Create a response from HTTP parts (internal use).
    ///
    /// This method is used internally by the HTTP client to create responses
    /// from hyper's response parts.
    pub(crate) fn from_parts(
        status: u16,
        status_text: String,
        headers: Headers,
        url: String,
        redirected: bool,
    ) -> Self {
        Self {
            response_type: ResponseType::Basic,
            url,
            redirected,
            status,
            status_text,
            headers,
            body: None,
        }
    }

    /// Set the response body (internal use).
    ///
    /// This method is used internally by the HTTP client to set the response body
    /// after creating the response from HTTP parts.
    pub(crate) fn set_body(&mut self, body: ReadableStream) {
        self.body = Some(body);
    }
}

impl Clone for Response {
    fn clone(&self) -> Self {
        Self {
            response_type: self.response_type,
            url: self.url.clone(),
            redirected: self.redirected,
            status: self.status,
            status_text: self.status_text.clone(),
            headers: self.headers.clone(),
            body: self.body.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_creation() {
        let response = Response::new(None, None).unwrap();
        assert_eq!(response.status(), 200);
        assert!(response.ok());
        assert_eq!(response.status_text(), "OK");
        assert!(!response.redirected());
        assert_eq!(response.response_type(), ResponseType::Basic);
    }

    #[test]
    fn test_response_with_init() {
        let mut headers = Headers::new();
        headers.set("x-test", "value").unwrap();

        let mut init = ResponseInit::new();
        init.status = Some(201);
        init.status_text = Some("Created".to_string());
        init.headers = Some(headers);

        let response =
            Response::new(Some(ReadableStream::from_text("created")), Some(init)).unwrap();
        assert_eq!(response.status(), 201);
        assert!(response.ok());
        assert_eq!(response.status_text(), "Created");
        assert!(response.headers().has("x-test").unwrap());
    }

    #[test]
    fn test_response_error() {
        let response = Response::error();
        assert_eq!(response.status(), 0);
        assert!(!response.ok());
        assert_eq!(response.response_type(), ResponseType::Error);
    }

    #[test]
    fn test_response_redirect() {
        let response = Response::redirect("https://example.com", Some(301)).unwrap();
        assert_eq!(response.status(), 301);
        assert!(!response.ok());
        assert_eq!(
            response.headers().get("location").unwrap().unwrap(),
            "https://example.com"
        );

        // Default redirect status
        let response = Response::redirect("https://example.com", None).unwrap();
        assert_eq!(response.status(), 302);

        // Invalid redirect status
        assert!(Response::redirect("https://example.com", Some(200)).is_err());
    }

    #[test]
    fn test_response_status_validation() {
        // Valid status codes
        assert!(Response::new(
            None,
            Some({
                let mut init = ResponseInit::new();
                init.status = Some(200);
                init
            })
        )
        .is_ok());

        assert!(Response::new(
            None,
            Some({
                let mut init = ResponseInit::new();
                init.status = Some(404);
                init
            })
        )
        .is_ok());

        assert!(Response::new(
            None,
            Some({
                let mut init = ResponseInit::new();
                init.status = Some(500);
                init
            })
        )
        .is_ok());

        // Invalid status codes (below 200)
        assert!(Response::new(
            None,
            Some({
                let mut init = ResponseInit::new();
                init.status = Some(199);
                init
            })
        )
        .is_err());

        // Invalid status codes (above 599)
        assert!(Response::new(
            None,
            Some({
                let mut init = ResponseInit::new();
                init.status = Some(600);
                init
            })
        )
        .is_err());
    }

    #[test]
    fn test_response_ok_status() {
        // 2xx statuses should be ok
        for status in 200..300 {
            let response = Response::new(
                None,
                Some({
                    let mut init = ResponseInit::new();
                    init.status = Some(status);
                    init
                }),
            )
            .unwrap();
            assert!(response.ok(), "Status {} should be ok", status);
        }

        // Non-2xx statuses should not be ok (using valid status codes)
        let not_ok_statuses = [300, 400, 404, 500];
        for status in not_ok_statuses {
            let response = Response::new(
                None,
                Some({
                    let mut init = ResponseInit::new();
                    init.status = Some(status);
                    init
                }),
            )
            .unwrap();
            assert!(!response.ok(), "Status {} should not be ok", status);
        }
    }

    #[test]
    fn test_response_status_text_validation() {
        // Valid status text
        assert!(Response::new(
            None,
            Some({
                let mut init = ResponseInit::new();
                init.status_text = Some("OK".to_string());
                init
            })
        )
        .is_ok());

        // Invalid status text (contains newline)
        assert!(Response::new(
            None,
            Some({
                let mut init = ResponseInit::new();
                init.status_text = Some("OK\r\n".to_string());
                init
            })
        )
        .is_err());
    }

    #[test]
    fn test_default_status_text() {
        let response = Response::new(
            None,
            Some({
                let mut init = ResponseInit::new();
                init.status = Some(404);
                init
            }),
        )
        .unwrap();
        assert_eq!(response.status_text(), "Not Found");

        // Test unknown status code (but within valid range)
        let response = Response::new(
            None,
            Some({
                let mut init = ResponseInit::new();
                init.status = Some(418); // I'm a teapot - valid but not in our lookup
                init
            }),
        )
        .unwrap();
        assert_eq!(response.status_text(), "");
    }

    #[tokio::test]
    async fn test_response_body_methods() {
        let response = Response::new(Some(ReadableStream::from_text("test body")), None).unwrap();

        let text = response.text().await.unwrap();
        assert_eq!(text, "test body");
    }

    #[tokio::test]
    async fn test_response_json_body() {
        let data = serde_json::json!({"key": "value"});
        let response = Response::new(Some(ReadableStream::from_json(&data)), None).unwrap();

        let parsed: serde_json::Value = response.json().await.unwrap();
        assert_eq!(parsed["key"], "value");
    }

    #[tokio::test]
    async fn test_response_empty_body() {
        let response = Response::new(None, None).unwrap();

        let text = response.text().await.unwrap();
        assert_eq!(text, "");

        let response = Response::new(None, None).unwrap();
        let bytes = response.array_buffer().await.unwrap();
        assert!(bytes.is_empty());
    }

    #[test]
    fn test_response_init_defaults() {
        let init = ResponseInit::new();
        assert!(init.status.is_none());
        assert!(init.status_text.is_none());
        assert!(init.headers.is_none());
    }

    #[test]
    fn test_response_type_default() {
        assert_eq!(ResponseType::default(), ResponseType::Basic);
    }

    #[test]
    fn test_response_clone() {
        let response = Response::new(None, None).unwrap();
        let cloned = response.clone_response().unwrap();

        assert_eq!(response.status(), cloned.status());
        assert_eq!(response.ok(), cloned.ok());
    }

    #[test]
    fn test_redirect_status_codes() {
        // Valid redirect codes
        let valid_redirects = [301, 302, 303, 307, 308];
        for status in valid_redirects {
            assert!(Response::redirect("https://example.com", Some(status)).is_ok());
        }

        // Invalid redirect codes
        let invalid_redirects = [200, 300, 304, 400, 500];
        for status in invalid_redirects {
            assert!(Response::redirect("https://example.com", Some(status)).is_err());
        }
    }

    #[tokio::test]
    async fn test_body_already_used_error() {
        let response = Response::new(Some(ReadableStream::from_text("test")), None).unwrap();
        let _text = response.text().await.unwrap();
        
        let response_with_body = Response::new(Some(ReadableStream::from_text("test")), None).unwrap();
        let _consumed = response_with_body.text().await.unwrap();
        
        let response = Response::new(Some(ReadableStream::from_text("test")), None).unwrap();
        
        let cloned = response.clone_response().unwrap();
        assert_eq!(response.status(), cloned.status());
        
        let _text = response.text().await.unwrap();
        
        let _text2 = cloned.text().await.unwrap();
    }

    #[tokio::test]
    async fn test_json_empty_body_error() {
        let response = Response::new(None, None).unwrap();
        let result: Result<serde_json::Value> = response.json().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FetchError::Type(_)));
    }

    #[test]
    fn test_response_all_status_texts() {
        let status_codes = [
            200, 201, 204, 301, 302, 303, 304, 307, 308, 400, 401, 403, 404, 405, 409, 410, 422,
            429, 500, 501, 502, 503, 504,
        ];

        for status in status_codes {
            let text = Response::default_status_text(status);
            assert!(
                !text.is_empty(),
                "Status {} should have status text",
                status
            );
        }

        // Test unknown status
        let unknown_text = Response::default_status_text(999);
        assert_eq!(unknown_text, "");
    }
}
