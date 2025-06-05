//! HTTP request types and builders following the WHATWG Fetch specification.
//!
//! This module provides the [`Request`] type and related configuration types for
//! creating and managing HTTP requests. It follows the WHATWG Fetch specification
//! closely to provide a familiar API for web developers.
//!
//! # Request Creation
//!
//! Requests are created using the [`Request::new()`] constructor, which takes a URL
//! and optional [`RequestInit`] configuration. The configuration allows setting
//! HTTP method, headers, body, and various fetch options.
//!
//! # Examples
//!
//! ## Simple GET Request
//!
//! ```rust
//! use fetch::Request;
//!
//! let request = Request::new("https://api.example.com/users", None).unwrap();
//! assert_eq!(request.method(), "GET");
//! assert_eq!(request.url(), "https://api.example.com/users");
//! ```
//!
//! ## POST Request with JSON Body
//!
//! ```rust
//! use fetch::{Request, RequestInit, ReadableStream};
//! use serde_json::json;
//!
//! let data = json!({"name": "John", "email": "john@example.com"});
//!
//! let mut init = RequestInit::new();
//! init.method = Some("POST".to_string());
//! init.body = Some(ReadableStream::from_json(&data));
//!
//! let request = Request::new("https://api.example.com/users", Some(init)).unwrap();
//! assert_eq!(request.method(), "POST");
//! assert!(request.body().is_some());
//! ```
//!
//! ## Request with Custom Headers
//!
//! ```rust
//! use fetch::{Request, RequestInit, Headers};
//!
//! let mut headers = Headers::new();
//! headers.set("Authorization", "Bearer token123").unwrap();
//! headers.set("X-API-Version", "v1").unwrap();
//!
//! let mut init = RequestInit::new();
//! init.headers = Some(headers);
//!
//! let request = Request::new("https://api.example.com/protected", Some(init)).unwrap();
//! assert!(request.headers().has("authorization").unwrap());
//! ```

use crate::error::{FetchError, Result, TypeError};
use crate::{AbortSignal, Headers, ReadableStream};
use url::Url;

/// CORS mode for requests.
///
/// This enum specifies how cross-origin requests should be handled, following
/// the WHATWG Fetch specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestMode {
    /// Only allow same-origin requests
    SameOrigin,
    /// Use CORS for cross-origin requests (default)
    Cors,
    /// Allow cross-origin requests without CORS
    NoCors,
    /// Navigation requests (for HTML documents)
    Navigate,
}

impl Default for RequestMode {
    fn default() -> Self {
        Self::Cors
    }
}

/// Credentials mode for requests.
///
/// This enum controls whether credentials (cookies, authorization headers, etc.)
/// are included in requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestCredentials {
    /// Never include credentials
    Omit,
    /// Include credentials for same-origin requests only (default)
    SameOrigin,
    /// Always include credentials
    Include,
}

impl Default for RequestCredentials {
    fn default() -> Self {
        Self::SameOrigin
    }
}

/// Cache mode for requests.
///
/// This enum controls how the request interacts with the HTTP cache.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestCache {
    /// Use default cache behavior
    Default,
    /// Don't use cache, don't store response
    NoStore,
    /// Bypass cache, always fetch from network
    Reload,
    /// Bypass cache but store response
    NoCache,
    /// Use cache if possible, don't validate
    ForceCache,
    /// Only use cache, fail if not cached
    OnlyIfCached,
}

impl Default for RequestCache {
    fn default() -> Self {
        Self::Default
    }
}

/// Redirect mode for requests.
///
/// This enum controls how redirects are handled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestRedirect {
    /// Follow redirects automatically (default)
    Follow,
    /// Fail on redirects
    Error,
    /// Return redirect response without following
    Manual,
}

impl Default for RequestRedirect {
    fn default() -> Self {
        Self::Follow
    }
}

/// Configuration for creating requests.
///
/// `RequestInit` provides all the options that can be set when creating a new
/// [`Request`]. All fields are optional and will use defaults if not specified.
///
/// # Examples
///
/// ```rust
/// use fetch::{RequestInit, ReadableStream, Headers, RequestMode};
///
/// let mut init = RequestInit::new();
/// init.method = Some("PUT".to_string());
/// init.body = Some(ReadableStream::from_text("Hello, World!"));
/// init.mode = Some(RequestMode::Cors);
///
/// // Headers can be set using the Headers type
/// let mut headers = Headers::new();
/// headers.set("Content-Type", "text/plain").unwrap();
/// init.headers = Some(headers);
/// ```
#[derive(Debug, Clone, Default)]
pub struct RequestInit {
    /// HTTP method (GET, POST, PUT, etc.)
    pub method: Option<String>,
    /// Request headers
    pub headers: Option<Headers>,
    /// Request body
    pub body: Option<ReadableStream>,
    /// CORS mode
    pub mode: Option<RequestMode>,
    /// Credentials mode
    pub credentials: Option<RequestCredentials>,
    /// Cache mode
    pub cache: Option<RequestCache>,
    /// Redirect mode
    pub redirect: Option<RequestRedirect>,
    /// Referrer URL or policy
    pub referrer: Option<String>,
    /// Referrer policy
    pub referrer_policy: Option<String>,
    /// Subresource integrity metadata
    pub integrity: Option<String>,
    /// Keep connection alive after page unload
    pub keepalive: Option<bool>,
    /// Abort signal for cancellation
    pub signal: Option<AbortSignal>,
}

impl RequestInit {
    /// Create a new empty RequestInit.
    ///
    /// All fields will be `None` and will use defaults when creating a request.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetch::RequestInit;
    ///
    /// let init = RequestInit::new();
    /// assert!(init.method.is_none());
    /// assert!(init.headers.is_none());
    /// assert!(init.body.is_none());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }
}

/// An HTTP request following the WHATWG Fetch specification.
///
/// `Request` represents an HTTP request with all its associated metadata including
/// URL, method, headers, body, and fetch options. Requests are immutable once
/// created, but can be cloned if the body hasn't been consumed.
///
/// # Body Consumption
///
/// Request bodies can only be consumed once. After calling any body consumption
/// method (`text()`, `json()`, `array_buffer()`, etc.), the body is marked as used
/// and cannot be consumed again.
///
/// # Examples
///
/// ```rust
/// use fetch::{Request, RequestInit};
///
/// // Simple GET request
/// let request = Request::new("https://api.example.com/data", None).unwrap();
/// println!("Method: {}", request.method());
/// println!("URL: {}", request.url());
///
/// // POST request with configuration
/// let mut init = RequestInit::new();
/// init.method = Some("POST".to_string());
///
/// let request = Request::new("https://api.example.com/submit", Some(init)).unwrap();
/// assert_eq!(request.method(), "POST");
/// ```
#[derive(Debug, Clone)]
pub struct Request {
    /// Parsed URL for the request
    url: Url,
    /// HTTP method (normalized to uppercase for standard methods)
    method: String,
    /// Request headers
    headers: Headers,
    /// Request body (optional)
    body: Option<ReadableStream>,
    /// CORS mode
    mode: RequestMode,
    /// Credentials mode
    credentials: RequestCredentials,
    /// Cache mode
    cache: RequestCache,
    /// Redirect mode
    redirect: RequestRedirect,
    /// Referrer information
    referrer: String,
    /// Referrer policy
    referrer_policy: String,
    /// Subresource integrity metadata
    integrity: String,
    /// Keep-alive flag
    keepalive: bool,
    /// Abort signal for cancellation
    signal: Option<AbortSignal>,
}

impl Request {
    /// Create a new HTTP request.
    ///
    /// This constructor validates the URL and request configuration, applies
    /// defaults for unspecified options, and performs validation according to
    /// the WHATWG Fetch specification.
    ///
    /// # Arguments
    ///
    /// * `input` - The URL to request (must be a valid absolute URL)
    /// * `init` - Optional request configuration
    ///
    /// # Returns
    ///
    /// A new `Request` instance, or an error if validation fails.
    ///
    /// # Errors
    ///
    /// * [`TypeError`] - If the URL is invalid, method is invalid, or GET/HEAD requests have a body
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetch::{Request, RequestInit, ReadableStream};
    ///
    /// // Simple GET request
    /// let request = Request::new("https://example.com", None).unwrap();
    /// assert_eq!(request.method(), "GET");
    ///
    /// // POST request with body
    /// let mut init = RequestInit::new();
    /// init.method = Some("POST".to_string());
    /// init.body = Some(ReadableStream::from_text("data"));
    ///
    /// let request = Request::new("https://example.com/api", Some(init)).unwrap();
    /// assert_eq!(request.method(), "POST");
    ///
    /// // Invalid URL will fail
    /// assert!(Request::new("not-a-url", None).is_err());
    ///
    /// // GET with body will fail
    /// let mut invalid_init = RequestInit::new();
    /// invalid_init.method = Some("GET".to_string());
    /// invalid_init.body = Some(ReadableStream::from_text("body"));
    /// assert!(Request::new("https://example.com", Some(invalid_init)).is_err());
    /// ```
    pub fn new(input: &str, init: Option<RequestInit>) -> Result<Self> {
        // Parse and validate URL
        let url = Url::parse(input)?;
        let init = init.unwrap_or_default();

        // Validate and normalize method
        let method = init.method.unwrap_or_else(|| "GET".to_string());
        let method = Self::normalize_method(&method)?;

        // Validate method-body combinations
        if matches!(method.as_str(), "GET" | "HEAD") && init.body.is_some() {
            return Err(FetchError::Type(TypeError::new(
                "Request with GET/HEAD method cannot have body",
            )));
        }

        // Initialize headers
        let mut headers = init.headers.unwrap_or_default();

        // Auto-set Content-Type for bodies that have a default type
        if let Some(ref body) = init.body {
            if let (Ok(None), Some(content_type)) =
                (headers.get("content-type"), body.get_content_type())
            {
                headers.set("content-type", content_type)?;
            }
        }

        Ok(Self {
            url,
            method,
            headers,
            body: init.body,
            mode: init.mode.unwrap_or_default(),
            credentials: init.credentials.unwrap_or_default(),
            cache: init.cache.unwrap_or_default(),
            redirect: init.redirect.unwrap_or_default(),
            referrer: init.referrer.unwrap_or_else(|| "about:client".to_string()),
            referrer_policy: init.referrer_policy.unwrap_or_default(),
            integrity: init.integrity.unwrap_or_default(),
            keepalive: init.keepalive.unwrap_or(false),
            signal: init.signal,
        })
    }

    /// Get the request URL.
    ///
    /// # Returns
    ///
    /// The request URL as a string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetch::Request;
    ///
    /// let request = Request::new("https://api.example.com/users?page=1", None).unwrap();
    /// assert_eq!(request.url(), "https://api.example.com/users?page=1");
    /// ```
    pub fn url(&self) -> &str {
        self.url.as_str()
    }

    /// Get the request method.
    ///
    /// # Returns
    ///
    /// The HTTP method as a string (uppercase for standard methods).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetch::{Request, RequestInit};
    ///
    /// let request = Request::new("https://example.com", None).unwrap();
    /// assert_eq!(request.method(), "GET");
    ///
    /// let mut init = RequestInit::new();
    /// init.method = Some("post".to_string()); // Will be normalized to uppercase
    /// let request = Request::new("https://example.com", Some(init)).unwrap();
    /// assert_eq!(request.method(), "POST");
    /// ```
    pub fn method(&self) -> &str {
        &self.method
    }

    /// Get the request headers.
    ///
    /// # Returns
    ///
    /// A reference to the request headers.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetch::{Request, RequestInit, Headers};
    ///
    /// let mut headers = Headers::new();
    /// headers.set("User-Agent", "MyApp/1.0").unwrap();
    ///
    /// let mut init = RequestInit::new();
    /// init.headers = Some(headers);
    ///
    /// let request = Request::new("https://example.com", Some(init)).unwrap();
    /// assert!(request.headers().has("user-agent").unwrap());
    /// ```
    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    /// Get the request body.
    ///
    /// # Returns
    ///
    /// An optional reference to the request body.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetch::{Request, RequestInit, ReadableStream};
    ///
    /// // Request without body
    /// let request = Request::new("https://example.com", None).unwrap();
    /// assert!(request.body().is_none());
    ///
    /// // Request with body
    /// let mut init = RequestInit::new();
    /// init.method = Some("POST".to_string());
    /// init.body = Some(ReadableStream::from_text("data"));
    ///
    /// let request = Request::new("https://example.com", Some(init)).unwrap();
    /// assert!(request.body().is_some());
    /// ```
    pub fn body(&self) -> Option<&ReadableStream> {
        self.body.as_ref()
    }

    /// Check if the request body has been used.
    ///
    /// # Returns
    ///
    /// `true` if the body has been consumed, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetch::{Request, RequestInit, ReadableStream};
    ///
    /// let mut init = RequestInit::new();
    /// init.method = Some("POST".to_string());
    /// init.body = Some(ReadableStream::from_text("data"));
    ///
    /// let request = Request::new("https://example.com", Some(init)).unwrap();
    /// assert!(!request.body_used());
    ///
    /// // After consuming the body, it would be marked as used
    /// // (body consumption moves the request, so we can't show this in the example)
    /// ```
    pub fn body_used(&self) -> bool {
        self.body.as_ref().is_some_and(|b| b.is_used())
    }

    /// Get the request mode.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetch::{Request, RequestMode};
    ///
    /// let request = Request::new("https://example.com", None).unwrap();
    /// assert_eq!(request.mode(), RequestMode::Cors);
    /// ```
    pub fn mode(&self) -> RequestMode {
        self.mode
    }

    /// Get the credentials mode.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetch::{Request, RequestCredentials};
    ///
    /// let request = Request::new("https://example.com", None).unwrap();
    /// assert_eq!(request.credentials(), RequestCredentials::SameOrigin);
    /// ```
    pub fn credentials(&self) -> RequestCredentials {
        self.credentials
    }

    /// Get the cache mode.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetch::{Request, RequestCache};
    ///
    /// let request = Request::new("https://example.com", None).unwrap();
    /// assert_eq!(request.cache(), RequestCache::Default);
    /// ```
    pub fn cache(&self) -> RequestCache {
        self.cache
    }

    /// Get the redirect mode.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetch::{Request, RequestRedirect};
    ///
    /// let request = Request::new("https://example.com", None).unwrap();
    /// assert_eq!(request.redirect(), RequestRedirect::Follow);
    /// ```
    pub fn redirect(&self) -> RequestRedirect {
        self.redirect
    }

    /// Get the referrer information.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetch::Request;
    ///
    /// let request = Request::new("https://example.com", None).unwrap();
    /// assert_eq!(request.referrer(), "about:client");
    /// ```
    pub fn referrer(&self) -> &str {
        &self.referrer
    }

    /// Get the referrer policy.
    pub fn referrer_policy(&self) -> &str {
        &self.referrer_policy
    }

    /// Get the integrity metadata.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetch::Request;
    ///
    /// let request = Request::new("https://example.com", None).unwrap();
    /// assert_eq!(request.integrity(), "");
    /// ```
    pub fn integrity(&self) -> &str {
        &self.integrity
    }

    /// Get the keepalive flag.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetch::Request;
    ///
    /// let request = Request::new("https://example.com", None).unwrap();
    /// assert!(!request.keepalive());
    /// ```
    pub fn keepalive(&self) -> bool {
        self.keepalive
    }

    /// Get the abort signal.
    ///
    /// # Returns
    ///
    /// An optional reference to the abort signal.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetch::{Request, RequestInit, AbortController};
    ///
    /// let controller = AbortController::new();
    /// let mut init = RequestInit::new();
    /// init.signal = Some(controller.signal().clone());
    ///
    /// let request = Request::new("https://example.com", Some(init)).unwrap();
    /// assert!(request.signal().is_some());
    /// ```
    pub fn signal(&self) -> Option<&AbortSignal> {
        self.signal.as_ref()
    }

    /// Clone the request (WHATWG Fetch API method).
    ///
    /// This method follows the WHATWG Fetch specification for cloning requests.
    /// It will fail if the request body has already been used.
    ///
    /// # Returns
    ///
    /// A cloned request, or an error if the body has been used.
    ///
    /// # Errors
    ///
    /// * [`TypeError`] - If the request body has already been consumed
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetch::Request;
    ///
    /// let request = Request::new("https://example.com", None).unwrap();
    /// let cloned = request.clone_request().unwrap();
    ///
    /// assert_eq!(request.url(), cloned.url());
    /// assert_eq!(request.method(), cloned.method());
    /// ```
    pub fn clone_request(&self) -> Result<Self> {
        if self.body_used() {
            return Err(FetchError::Type(TypeError::new(
                "Cannot clone a request with a used body",
            )));
        }
        Ok(Clone::clone(self))
    }

    /// Consume the request and return the body as bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetch::{Request, RequestInit, ReadableStream};
    ///
    /// # tokio_test::block_on(async {
    /// let mut init = RequestInit::new();
    /// init.method = Some("POST".to_string());
    /// init.body = Some(ReadableStream::from_text("Hello, World!"));
    ///
    /// let request = Request::new("https://example.com", Some(init)).unwrap();
    /// let bytes = request.array_buffer().await.unwrap();
    /// assert_eq!(bytes, b"Hello, World!");
    /// # });
    /// ```
    pub async fn array_buffer(self) -> Result<bytes::Bytes> {
        match self.body {
            Some(body) => body.array_buffer().await,
            None => Ok(bytes::Bytes::new()),
        }
    }

    /// Consume the request and return the body as a blob (bytes).
    pub async fn blob(self) -> Result<bytes::Bytes> {
        self.array_buffer().await
    }

    /// Consume the request and return the body as form data.
    pub async fn form_data(self) -> Result<String> {
        match self.body {
            Some(body) => body.form_data().await,
            None => Ok(String::new()),
        }
    }

    /// Consume the request and parse the body as JSON.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetch::{Request, RequestInit, ReadableStream};
    /// use serde_json::json;
    ///
    /// # tokio_test::block_on(async {
    /// let data = json!({"name": "John", "age": 30});
    ///
    /// let mut init = RequestInit::new();
    /// init.method = Some("POST".to_string());
    /// init.body = Some(ReadableStream::from_json(&data));
    ///
    /// let request = Request::new("https://example.com", Some(init)).unwrap();
    /// let parsed: serde_json::Value = request.json().await.unwrap();
    /// assert_eq!(parsed["name"], "John");
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

    /// Consume the request and return the body as text.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetch::{Request, RequestInit, ReadableStream};
    ///
    /// # tokio_test::block_on(async {
    /// let mut init = RequestInit::new();
    /// init.method = Some("POST".to_string());
    /// init.body = Some(ReadableStream::from_text("Hello, World!"));
    ///
    /// let request = Request::new("https://example.com", Some(init)).unwrap();
    /// let text = request.text().await.unwrap();
    /// assert_eq!(text, "Hello, World!");
    /// # });
    /// ```
    pub async fn text(self) -> Result<String> {
        match self.body {
            Some(body) => body.text().await,
            None => Ok(String::new()),
        }
    }

    /// Normalize and validate an HTTP method.
    ///
    /// Standard methods (GET, POST, etc.) are normalized to uppercase.
    /// Custom methods are preserved as-is but validated for valid characters.
    fn normalize_method(method: &str) -> Result<String> {
        if method.is_empty() {
            return Err(FetchError::Type(TypeError::new("Invalid method")));
        }

        // Validate method characters (HTTP token)
        for byte in method.bytes() {
            if !matches!(byte, b'!' | b'#'..=b'\'' | b'*' | b'+' | b'-' | b'.' | b'0'..=b'9' | b'A'..=b'Z' | b'^'..=b'z' | b'|' | b'~')
            {
                return Err(FetchError::Type(TypeError::new("Invalid method")));
            }
        }

        // Normalize standard methods to uppercase
        let upper = method.to_ascii_uppercase();
        match upper.as_str() {
            "GET" | "POST" | "PUT" | "DELETE" | "HEAD" | "OPTIONS" | "PATCH" => Ok(upper),
            _ => Ok(method.to_string()), // Preserve case for custom methods
        }
    }

    /// Get the internal URL object for use by the client.
    pub(crate) fn get_url(&self) -> &Url {
        &self.url
    }

    /// Take the body from the request for consumption.
    ///
    /// This method is used internally by the HTTP client to consume the request body.
    pub(crate) fn take_body(&mut self) -> Option<ReadableStream> {
        self.body.take()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_creation() {
        let request = Request::new("https://example.com", None).unwrap();
        assert_eq!(request.url(), "https://example.com/");
        assert_eq!(request.method(), "GET");
        assert_eq!(request.mode(), RequestMode::Cors);
        assert_eq!(request.credentials(), RequestCredentials::SameOrigin);
        assert_eq!(request.cache(), RequestCache::Default);
        assert_eq!(request.redirect(), RequestRedirect::Follow);
        assert!(!request.keepalive());
        assert!(!request.body_used());
    }

    #[test]
    fn test_request_with_init() {
        let mut headers = Headers::new();
        headers.set("x-test", "value").unwrap();

        let mut init = RequestInit::new();
        init.method = Some("POST".to_string());
        init.headers = Some(headers);
        init.body = Some(ReadableStream::from_text("test body"));
        init.mode = Some(RequestMode::SameOrigin);
        init.credentials = Some(RequestCredentials::Include);

        let request = Request::new("https://example.com", Some(init)).unwrap();
        assert_eq!(request.method(), "POST");
        assert_eq!(request.mode(), RequestMode::SameOrigin);
        assert_eq!(request.credentials(), RequestCredentials::Include);
        assert!(request.headers().has("x-test").unwrap());
        assert!(request.body().is_some());
    }

    #[test]
    fn test_request_method_validation() {
        // Valid methods
        assert!(Request::new(
            "https://example.com",
            Some({
                let mut init = RequestInit::new();
                init.method = Some("GET".to_string());
                init
            })
        )
        .is_ok());

        assert!(Request::new(
            "https://example.com",
            Some({
                let mut init = RequestInit::new();
                init.method = Some("POST".to_string());
                init
            })
        )
        .is_ok());

        assert!(Request::new(
            "https://example.com",
            Some({
                let mut init = RequestInit::new();
                init.method = Some("CUSTOM".to_string());
                init
            })
        )
        .is_ok());

        // Invalid method (empty)
        assert!(Request::new(
            "https://example.com",
            Some({
                let mut init = RequestInit::new();
                init.method = Some("".to_string());
                init
            })
        )
        .is_err());

        // GET with body should fail
        assert!(Request::new(
            "https://example.com",
            Some({
                let mut init = RequestInit::new();
                init.method = Some("GET".to_string());
                init.body = Some(ReadableStream::from_text("body"));
                init
            })
        )
        .is_err());
    }

    #[test]
    fn test_request_url_validation() {
        // Valid URLs
        assert!(Request::new("https://example.com", None).is_ok());
        assert!(Request::new("http://localhost:8080/path", None).is_ok());

        // Invalid URLs
        assert!(Request::new("not-a-url", None).is_err());
        assert!(Request::new("", None).is_err());
    }

    #[test]
    fn test_request_defaults() {
        let init = RequestInit::new();
        assert!(init.method.is_none());
        assert!(init.headers.is_none());
        assert!(init.body.is_none());
        assert!(init.mode.is_none());
        assert!(init.credentials.is_none());
        assert!(init.cache.is_none());
        assert!(init.redirect.is_none());
        assert!(init.referrer.is_none());
        assert!(init.referrer_policy.is_none());
        assert!(init.integrity.is_none());
        assert!(init.keepalive.is_none());
        assert!(init.signal.is_none());
    }

    #[test]
    fn test_request_enum_defaults() {
        assert_eq!(RequestMode::default(), RequestMode::Cors);
        assert_eq!(
            RequestCredentials::default(),
            RequestCredentials::SameOrigin
        );
        assert_eq!(RequestCache::default(), RequestCache::Default);
        assert_eq!(RequestRedirect::default(), RequestRedirect::Follow);
    }

    #[tokio::test]
    async fn test_request_body_methods() {
        let request = Request::new(
            "https://example.com",
            Some({
                let mut init = RequestInit::new();
                init.method = Some("POST".to_string());
                init.body = Some(ReadableStream::from_text("test body"));
                init
            }),
        )
        .unwrap();

        let text = request.text().await.unwrap();
        assert_eq!(text, "test body");
    }

    #[tokio::test]
    async fn test_request_json_body() {
        let data = serde_json::json!({"key": "value"});
        let request = Request::new(
            "https://example.com",
            Some({
                let mut init = RequestInit::new();
                init.method = Some("POST".to_string());
                init.body = Some(ReadableStream::from_json(&data));
                init
            }),
        )
        .unwrap();

        let parsed: serde_json::Value = request.json().await.unwrap();
        assert_eq!(parsed["key"], "value");
    }

    #[test]
    fn test_method_normalization() {
        let request = Request::new(
            "https://example.com",
            Some({
                let mut init = RequestInit::new();
                init.method = Some("get".to_string());
                init
            }),
        )
        .unwrap();
        assert_eq!(request.method(), "GET");

        let request = Request::new(
            "https://example.com",
            Some({
                let mut init = RequestInit::new();
                init.method = Some("custom".to_string());
                init
            }),
        )
        .unwrap();
        assert_eq!(request.method(), "custom");
    }

    #[test]
    fn test_content_type_auto_set() {
        let request = Request::new(
            "https://example.com",
            Some({
                let mut init = RequestInit::new();
                init.method = Some("POST".to_string());
                init.body = Some(ReadableStream::from_json(&serde_json::json!({})));
                init
            }),
        )
        .unwrap();

        assert_eq!(
            request.headers().get("content-type").unwrap().unwrap(),
            "application/json"
        );
    }

    #[test]
    fn test_request_clone() {
        let request = Request::new("https://example.com", None).unwrap();
        let cloned = request.clone_request().unwrap();

        assert_eq!(request.url(), cloned.url());
        assert_eq!(request.method(), cloned.method());
    }
}
