//! Request and response body handling with streaming support.
//!
//! This module provides the [`ReadableStream`] type for handling HTTP request and response
//! bodies. It supports various body types including text, bytes, JSON, and provides
//! streaming capabilities following the WHATWG Fetch specification.
//!
//! # Body Types
//!
//! The module supports several body source types:
//! - **Empty**: No body content
//! - **Text**: UTF-8 text content  
//! - **Bytes**: Raw binary data
//! - **JSON**: Structured data serialized as JSON
//!
//! # Usage Examples
//!
//! ```rust
//! use fetchttp::ReadableStream;
//! use serde_json::json;
//!
//! // Create different body types
//! let text_body = ReadableStream::from_text("Hello, World!");
//! let json_body = ReadableStream::from_json(&json!({"key": "value"}));
//! let bytes_body = ReadableStream::from_bytes(b"binary data".to_vec().into());
//!
//! // Consume bodies
//! # tokio_test::block_on(async {
//! let text = text_body.text().await.unwrap();
//! let data: serde_json::Value = json_body.json().await.unwrap();
//! let bytes = bytes_body.array_buffer().await.unwrap();
//! # });
//! ```

use crate::error::{FetchError, Result, TypeError};
use bytes::Bytes;
use serde_json::Value;

/// Internal representation of body data sources.
///
/// This enum represents the different types of data that can be used as
/// request or response bodies. Each variant stores the data in its most
/// appropriate format for efficient processing.
#[derive(Debug, Clone)]
pub enum BodySource {
    /// No body content
    Empty,
    /// UTF-8 text content
    Text(String),
    /// Raw binary data
    Bytes(Bytes),
    /// Structured JSON data
    Json(Value),
}

/// A readable stream representing request or response body data.
///
/// `ReadableStream` provides a unified interface for handling different types of
/// body content in HTTP requests and responses. It follows the WHATWG Fetch
/// specification for body handling, including the "used" flag to prevent
/// multiple consumption of the same body.
///
/// # Body Consumption
///
/// Each body can only be consumed once. After calling any of the consumption
/// methods (`text()`, `json()`, `array_buffer()`, etc.), the body is marked
/// as used and subsequent calls will return an error.
///
/// # Examples
///
/// ```rust
/// use fetchttp::ReadableStream;
///
/// # tokio_test::block_on(async {
/// // Create and consume a text body
/// let stream = ReadableStream::from_text("Hello, World!");
/// let text = stream.text().await.unwrap();
/// assert_eq!(text, "Hello, World!");
///
/// // Create and consume a JSON body
/// let data = serde_json::json!({"name": "John", "age": 30});
/// let stream = ReadableStream::from_json(&data);
/// let parsed: serde_json::Value = stream.json().await.unwrap();
/// assert_eq!(parsed["name"], "John");
/// # });
/// ```
#[derive(Debug, Clone)]
pub struct ReadableStream {
    /// The source data for this stream
    source: BodySource,
    /// Whether this stream has been consumed
    used: bool,
}

impl ReadableStream {
    /// Create an empty readable stream.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetchttp::ReadableStream;
    ///
    /// let stream = ReadableStream::empty();
    /// # tokio_test::block_on(async {
    /// let text = stream.text().await.unwrap();
    /// assert_eq!(text, "");
    /// # });
    /// ```
    pub fn empty() -> Self {
        Self {
            source: BodySource::Empty,
            used: false,
        }
    }

    /// Create a readable stream from text content.
    ///
    /// The text will be encoded as UTF-8 when converted to bytes.
    ///
    /// # Arguments
    ///
    /// * `text` - The text content for the stream
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetchttp::ReadableStream;
    ///
    /// let stream = ReadableStream::from_text("Hello, World!");
    /// # tokio_test::block_on(async {
    /// let content = stream.text().await.unwrap();
    /// assert_eq!(content, "Hello, World!");
    /// # });
    /// ```
    pub fn from_text(text: &str) -> Self {
        Self {
            source: BodySource::Text(text.to_string()),
            used: false,
        }
    }

    /// Create a readable stream from binary data.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The binary data for the stream
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetchttp::ReadableStream;
    /// use bytes::Bytes;
    ///
    /// let data = Bytes::from(b"binary data".to_vec());
    /// let stream = ReadableStream::from_bytes(data);
    /// # tokio_test::block_on(async {
    /// let bytes = stream.array_buffer().await.unwrap();
    /// assert_eq!(bytes, b"binary data");
    /// # });
    /// ```
    pub fn from_bytes(bytes: Bytes) -> Self {
        Self {
            source: BodySource::Bytes(bytes),
            used: false,
        }
    }

    /// Create a readable stream from JSON data.
    ///
    /// The JSON value will be serialized when the stream is consumed.
    /// This automatically sets the appropriate content type for HTTP requests.
    ///
    /// # Arguments
    ///
    /// * `value` - The JSON value for the stream
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetchttp::ReadableStream;
    /// use serde_json::json;
    ///
    /// let data = json!({"name": "Alice", "age": 25});
    /// let stream = ReadableStream::from_json(&data);
    /// # tokio_test::block_on(async {
    /// let parsed: serde_json::Value = stream.json().await.unwrap();
    /// assert_eq!(parsed["name"], "Alice");
    /// # });
    /// ```
    pub fn from_json(value: &Value) -> Self {
        Self {
            source: BodySource::Json(value.clone()),
            used: false,
        }
    }

    /// Check if the stream is locked.
    ///
    /// In this implementation, streams are never locked as we don't support
    /// multiple readers. This method exists for WHATWG Fetch API compatibility.
    ///
    /// # Returns
    ///
    /// Always returns `false` in this implementation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetchttp::ReadableStream;
    ///
    /// let stream = ReadableStream::from_text("test");
    /// assert!(!stream.locked());
    /// ```
    pub fn locked(&self) -> bool {
        false
    }

    /// Consume the stream and return the content as bytes.
    ///
    /// This method consumes the entire stream and returns the content as a
    /// `Bytes` object. After calling this method, the stream is marked as used.
    ///
    /// # Returns
    ///
    /// The stream content as bytes, or an error if the stream was already used
    /// or if serialization fails.
    ///
    /// # Errors
    ///
    /// * [`TypeError`] - If the stream was already used or JSON serialization fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetchttp::ReadableStream;
    ///
    /// # tokio_test::block_on(async {
    /// let stream = ReadableStream::from_text("Hello");
    /// let bytes = stream.array_buffer().await.unwrap();
    /// assert_eq!(bytes, b"Hello");
    /// # });
    /// ```
    pub async fn array_buffer(mut self) -> Result<Bytes> {
        if self.used {
            return Err(FetchError::Type(TypeError::new("Body already used")));
        }
        self.used = true;

        match self.source {
            BodySource::Empty => Ok(Bytes::new()),
            BodySource::Text(text) => Ok(Bytes::from(text.into_bytes())),
            BodySource::Bytes(bytes) => Ok(bytes),
            BodySource::Json(value) => {
                let vec = serde_json::to_vec(&value)?;
                Ok(Bytes::from(vec))
            }
        }
    }

    /// Consume the stream and return the content as a blob (bytes).
    ///
    /// This is an alias for [`array_buffer()`] and exists for WHATWG Fetch API
    /// compatibility. In web browsers, blobs and array buffers are different,
    /// but in this implementation they are the same.
    ///
    /// [`array_buffer()`]: ReadableStream::array_buffer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetchttp::ReadableStream;
    ///
    /// # tokio_test::block_on(async {
    /// let stream = ReadableStream::from_bytes(b"data".to_vec().into());
    /// let blob = stream.blob().await.unwrap();
    /// assert_eq!(blob, b"data");
    /// # });
    /// ```
    pub async fn blob(self) -> Result<Bytes> {
        self.array_buffer().await
    }

    /// Consume the stream and return the content as form data.
    ///
    /// Currently this is implemented as an alias for [`text()`] since we don't
    /// have specialized form data parsing. This exists for WHATWG Fetch API
    /// compatibility.
    ///
    /// [`text()`]: ReadableStream::text
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetchttp::ReadableStream;
    ///
    /// # tokio_test::block_on(async {
    /// let stream = ReadableStream::from_text("key=value&foo=bar");
    /// let form_data = stream.form_data().await.unwrap();
    /// assert_eq!(form_data, "key=value&foo=bar");
    /// # });
    /// ```
    pub async fn form_data(self) -> Result<String> {
        self.text().await
    }

    /// Consume the stream and parse the content as JSON.
    ///
    /// This method deserializes the stream content as JSON into the specified
    /// type. The type must implement [`serde::de::DeserializeOwned`].
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type to deserialize the JSON into
    ///
    /// # Returns
    ///
    /// The deserialized JSON data, or an error if the stream was already used
    /// or if JSON parsing fails.
    ///
    /// # Errors
    ///
    /// * [`TypeError`] - If the stream was already used or JSON parsing fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetchttp::ReadableStream;
    /// use serde_json::json;
    /// use serde::Deserialize;
    ///
    /// #[derive(Deserialize, Debug, PartialEq)]
    /// struct User {
    ///     name: String,
    ///     age: u32,
    /// }
    ///
    /// # tokio_test::block_on(async {
    /// let data = json!({"name": "Alice", "age": 30});
    /// let stream = ReadableStream::from_json(&data);
    ///
    /// let user: User = stream.json().await.unwrap();
    /// assert_eq!(user.name, "Alice");
    /// assert_eq!(user.age, 30);
    /// # });
    /// ```
    pub async fn json<T: serde::de::DeserializeOwned>(mut self) -> Result<T> {
        if self.used {
            return Err(FetchError::Type(TypeError::new("Body already used")));
        }
        self.used = true;

        match self.source {
            BodySource::Empty => Err(FetchError::Type(TypeError::new(
                "Unexpected end of JSON input",
            ))),
            BodySource::Text(text) => Ok(serde_json::from_str(&text)?),
            BodySource::Bytes(bytes) => Ok(serde_json::from_slice(&bytes)?),
            BodySource::Json(value) => Ok(serde_json::from_value(value)?),
        }
    }

    /// Consume the stream and return the content as text.
    ///
    /// This method consumes the entire stream and returns the content as a
    /// UTF-8 string. For binary data, this will attempt UTF-8 decoding.
    ///
    /// # Returns
    ///
    /// The stream content as a string, or an error if the stream was already
    /// used or if UTF-8 decoding fails.
    ///
    /// # Errors
    ///
    /// * [`TypeError`] - If the stream was already used or UTF-8 decoding fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetchttp::ReadableStream;
    ///
    /// # tokio_test::block_on(async {
    /// let stream = ReadableStream::from_text("Hello, World!");
    /// let text = stream.text().await.unwrap();
    /// assert_eq!(text, "Hello, World!");
    /// # });
    /// ```
    pub async fn text(mut self) -> Result<String> {
        if self.used {
            return Err(FetchError::Type(TypeError::new("Body already used")));
        }
        self.used = true;

        match self.source {
            BodySource::Empty => Ok(String::new()),
            BodySource::Text(text) => Ok(text),
            BodySource::Bytes(bytes) => String::from_utf8(bytes.to_vec())
                .map_err(|_| FetchError::Type(TypeError::new("Invalid UTF-8"))),
            BodySource::Json(value) => Ok(serde_json::to_string(&value)?),
        }
    }

    /// Get the appropriate Content-Type header value for this body.
    ///
    /// This method returns the MIME type that should be used in the
    /// Content-Type header for HTTP requests with this body.
    ///
    /// # Returns
    ///
    /// * `Some("text/plain;charset=UTF-8")` for text bodies
    /// * `Some("application/json")` for JSON bodies  
    /// * `None` for empty or binary bodies
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetchttp::ReadableStream;
    /// use serde_json::json;
    ///
    /// let text_body = ReadableStream::from_text("hello");
    /// assert_eq!(text_body.get_content_type(), Some("text/plain;charset=UTF-8"));
    ///
    /// let json_body = ReadableStream::from_json(&json!({}));
    /// assert_eq!(json_body.get_content_type(), Some("application/json"));
    ///
    /// let empty_body = ReadableStream::empty();
    /// assert_eq!(empty_body.get_content_type(), None);
    /// ```
    pub(crate) fn get_content_type(&self) -> Option<&'static str> {
        match self.source {
            BodySource::Empty => None,
            BodySource::Text(_) => Some("text/plain;charset=UTF-8"),
            BodySource::Bytes(_) => None,
            BodySource::Json(_) => Some("application/json"),
        }
    }

    /// Convert the body content to bytes without consuming the stream.
    ///
    /// This method is used internally to get the byte representation of the
    /// body content for HTTP transmission. Unlike `array_buffer()`, this
    /// method doesn't mark the stream as used.
    pub(crate) async fn to_bytes(&self) -> Result<Bytes> {
        match &self.source {
            BodySource::Empty => Ok(Bytes::new()),
            BodySource::Text(text) => Ok(Bytes::from(text.as_bytes().to_vec())),
            BodySource::Bytes(bytes) => Ok(bytes.clone()),
            BodySource::Json(value) => {
                let vec = serde_json::to_vec(value)?;
                Ok(Bytes::from(vec))
            }
        }
    }

    /// Check if the stream has been used.
    ///
    /// This method returns `true` if any of the consumption methods have been
    /// called on this stream.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetchttp::ReadableStream;
    ///
    /// # tokio_test::block_on(async {
    /// let stream = ReadableStream::from_text("test");
    /// assert!(!stream.is_used());
    ///
    /// let _text = stream.text().await.unwrap();
    /// // Note: stream is moved, so we can't check is_used() here
    /// # });
    /// ```
    pub(crate) fn is_used(&self) -> bool {
        self.used
    }
}

// Convenient conversion implementations
impl From<&str> for ReadableStream {
    fn from(text: &str) -> Self {
        Self::from_text(text)
    }
}

impl From<String> for ReadableStream {
    fn from(text: String) -> Self {
        Self::from_text(&text)
    }
}

impl From<Bytes> for ReadableStream {
    fn from(bytes: Bytes) -> Self {
        Self::from_bytes(bytes)
    }
}

impl From<Vec<u8>> for ReadableStream {
    fn from(bytes: Vec<u8>) -> Self {
        Self::from_bytes(Bytes::from(bytes))
    }
}

impl From<Value> for ReadableStream {
    fn from(value: Value) -> Self {
        Self::from_json(&value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_readable_stream_text() {
        let stream = ReadableStream::from_text("Hello, World!");
        let text = stream.text().await.unwrap();
        assert_eq!(text, "Hello, World!");
    }

    #[tokio::test]
    async fn test_readable_stream_bytes() {
        let data = vec![1, 2, 3, 4, 5];
        let stream = ReadableStream::from_bytes(Bytes::from(data.clone()));
        let bytes = stream.array_buffer().await.unwrap();
        assert_eq!(bytes.to_vec(), data);
    }

    #[tokio::test]
    async fn test_readable_stream_json() {
        let value = serde_json::json!({"key": "value", "number": 42});
        let stream = ReadableStream::from_json(&value);
        let parsed: serde_json::Value = stream.json().await.unwrap();
        assert_eq!(parsed["key"], "value");
        assert_eq!(parsed["number"], 42);
    }

    #[tokio::test]
    async fn test_readable_stream_empty() {
        let stream = ReadableStream::empty();
        let text = stream.text().await.unwrap();
        assert_eq!(text, "");
    }

    #[tokio::test]
    async fn test_readable_stream_blob() {
        let data = vec![1, 2, 3, 4];
        let stream = ReadableStream::from_bytes(Bytes::from(data.clone()));
        let blob = stream.blob().await.unwrap();
        assert_eq!(blob.to_vec(), data);
    }

    #[tokio::test]
    async fn test_readable_stream_form_data() {
        let stream = ReadableStream::from_text("form data");
        let form = stream.form_data().await.unwrap();
        assert_eq!(form, "form data");
    }

    #[test]
    fn test_readable_stream_locked() {
        let stream = ReadableStream::from_text("test");
        assert!(!stream.locked());
    }

    #[test]
    fn test_readable_stream_from_conversions() {
        let _stream1 = ReadableStream::from("text");
        let _stream2 = ReadableStream::from(String::from("text"));
        let _stream3 = ReadableStream::from(Bytes::from(vec![1, 2, 3]));
        let _stream4 = ReadableStream::from(vec![1, 2, 3]);
        let _stream5 = ReadableStream::from(serde_json::json!({"key": "value"}));
    }

    #[test]
    fn test_get_content_type() {
        let empty = ReadableStream::empty();
        assert_eq!(empty.get_content_type(), None);

        let text = ReadableStream::from_text("hello");
        assert_eq!(text.get_content_type(), Some("text/plain;charset=UTF-8"));

        let bytes = ReadableStream::from_bytes(Bytes::from(vec![1, 2, 3]));
        assert_eq!(bytes.get_content_type(), None);

        let json = ReadableStream::from_json(&serde_json::json!({}));
        assert_eq!(json.get_content_type(), Some("application/json"));
    }

    #[tokio::test]
    async fn test_to_bytes() {
        let text = ReadableStream::from_text("hello");
        let bytes = text.to_bytes().await.unwrap();
        assert_eq!(bytes, "hello".as_bytes());

        let json = ReadableStream::from_json(&serde_json::json!({"key": "value"}));
        let bytes = json.to_bytes().await.unwrap();
        let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(parsed["key"], "value");
    }

    #[tokio::test]
    async fn test_body_already_used_error() {
        let stream = ReadableStream::from_text("test");
        let _text = stream.text().await.unwrap();

        // Create a new stream to test the error
        let mut stream = ReadableStream::from_text("test");
        stream.used = true; // Manually mark as used for testing

        let result = stream.text().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FetchError::Type(_)));
    }

    #[tokio::test]
    async fn test_json_empty_body_error() {
        let stream = ReadableStream::empty();
        let result: Result<serde_json::Value> = stream.json().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FetchError::Type(_)));
    }

    #[tokio::test]
    async fn test_invalid_utf8_error() {
        // Create bytes that are not valid UTF-8
        let invalid_utf8 = vec![0xFF, 0xFE, 0xFD];
        let stream = ReadableStream::from_bytes(Bytes::from(invalid_utf8));

        let result = stream.text().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FetchError::Type(_)));
    }
}
