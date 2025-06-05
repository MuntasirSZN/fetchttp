//! Error types for the fetch library.
//!
//! This module provides comprehensive error handling following the WHATWG Fetch
//! specification. All errors implement standard Rust error traits and provide
//! detailed error information.

use std::fmt;

/// A type error indicating invalid arguments or operations.
///
/// This error type corresponds to JavaScript's `TypeError` and is used for
/// validation failures, invalid arguments, and other type-related errors.
///
/// # Examples
///
/// ```rust
/// use fetchttp::TypeError;
///
/// let error = TypeError::new("Invalid header name");
/// println!("Error: {}", error);
/// ```
#[derive(Debug, Clone)]
pub struct TypeError {
    message: String,
}

impl TypeError {
    /// Create a new TypeError with the given message.
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }

    /// Get the error message.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TypeError: {}", self.message)
    }
}

impl std::error::Error for TypeError {}

/// A network error indicating connection or protocol failures.
///
/// This error type represents network-level failures such as DNS resolution
/// errors, connection timeouts, TLS errors, and other transport-related issues.
///
/// # Examples
///
/// ```rust
/// use fetchttp::NetworkError;
///
/// let error = NetworkError::new("Connection refused");
/// println!("Error: {}", error);
/// ```
#[derive(Debug, Clone)]
pub struct NetworkError {
    message: String,
}

impl NetworkError {
    /// Create a new NetworkError with the given message.
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }

    /// Get the error message.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NetworkError: {}", self.message)
    }
}

impl std::error::Error for NetworkError {}

/// An abort error indicating the operation was cancelled.
///
/// This error type is used when a request is cancelled via an [`AbortSignal`].
/// It corresponds to JavaScript's `AbortError`.
///
/// [`AbortSignal`]: crate::AbortSignal
///
/// # Examples
///
/// ```rust
/// use fetchttp::AbortError;
///
/// let error = AbortError::new("The operation was aborted");
/// println!("Error: {}", error);
/// ```
#[derive(Debug, Clone)]
pub struct AbortError {
    message: String,
}

impl AbortError {
    /// Create a new AbortError with the given message.
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }

    /// Get the error message.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for AbortError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AbortError: {}", self.message)
    }
}

impl std::error::Error for AbortError {}

/// The main error type for fetch operations.
///
/// This enum encompasses all possible errors that can occur during fetch operations.
/// It provides a unified error type while preserving the specific error information.
///
/// # Variants
///
/// * [`Type`] - Type-related errors (invalid arguments, validation failures)
/// * [`Network`] - Network-related errors (connection, DNS, TLS failures)
/// * [`Abort`] - Request was aborted via abort signal
///
/// [`Type`]: FetchError::Type
/// [`Network`]: FetchError::Network
/// [`Abort`]: FetchError::Abort
///
/// # Examples
///
/// ```rust
/// use fetchttp::*;
///
/// async fn handle_fetch_error() {
///     match fetch("https://example.com", None).await {
///         Ok(response) => {
///             println!("Success: {}", response.status());
///         }
///         Err(FetchError::Type(e)) => {
///             eprintln!("Type error: {}", e);
///         }
///         Err(FetchError::Network(e)) => {
///             eprintln!("Network error: {}", e);
///         }
///         Err(FetchError::Abort(e)) => {
///             eprintln!("Request aborted: {}", e);
///         }
///     }
/// }
/// ```
#[derive(Debug)]
pub enum FetchError {
    /// Type-related error (invalid arguments, validation failures)
    Type(TypeError),
    /// Network-related error (connection, DNS, TLS failures)
    Network(NetworkError),
    /// Request was aborted
    Abort(AbortError),
}

impl fmt::Display for FetchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Type(e) => write!(f, "{}", e),
            Self::Network(e) => write!(f, "{}", e),
            Self::Abort(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for FetchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Type(e) => Some(e),
            Self::Network(e) => Some(e),
            Self::Abort(e) => Some(e),
        }
    }
}

// Conversion implementations for easy error handling
impl From<TypeError> for FetchError {
    fn from(err: TypeError) -> Self {
        Self::Type(err)
    }
}

impl From<NetworkError> for FetchError {
    fn from(err: NetworkError) -> Self {
        Self::Network(err)
    }
}

impl From<AbortError> for FetchError {
    fn from(err: AbortError) -> Self {
        Self::Abort(err)
    }
}

// Conversions from external error types
impl From<hyper::Error> for FetchError {
    fn from(err: hyper::Error) -> Self {
        Self::Network(NetworkError::new(&err.to_string()))
    }
}

impl From<hyper_util::client::legacy::Error> for FetchError {
    fn from(err: hyper_util::client::legacy::Error) -> Self {
        Self::Network(NetworkError::new(&err.to_string()))
    }
}

impl From<http::Error> for FetchError {
    fn from(_: http::Error) -> Self {
        Self::Network(NetworkError::new("HTTP error"))
    }
}

impl From<url::ParseError> for FetchError {
    fn from(_: url::ParseError) -> Self {
        Self::Type(TypeError::new("Invalid URL"))
    }
}

impl From<serde_json::Error> for FetchError {
    fn from(_: serde_json::Error) -> Self {
        Self::Type(TypeError::new("JSON parse error"))
    }
}

/// Convenient Result type alias for fetch operations.
///
/// This type alias provides a shorter way to write `Result<T, FetchError>`.
///
/// # Examples
///
/// ```rust
/// use fetchttp::{Result, Response};
///
/// async fn fetch_data() -> Result<Response> {
///     fetchttp::fetch("https://api.example.com/data", None).await
/// }
/// ```
pub type Result<T> = std::result::Result<T, FetchError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let type_error = TypeError::new("test message");
        assert_eq!(format!("{}", type_error), "TypeError: test message");

        let network_error = NetworkError::new("connection failed");
        assert_eq!(
            format!("{}", network_error),
            "NetworkError: connection failed"
        );

        let abort_error = AbortError::new("aborted");
        assert_eq!(format!("{}", abort_error), "AbortError: aborted");
    }

    #[test]
    fn test_fetch_error_conversions() {
        let type_error = TypeError::new("test");
        let fetch_error: FetchError = type_error.into();
        assert!(matches!(fetch_error, FetchError::Type(_)));

        let network_error = NetworkError::new("test");
        let fetch_error: FetchError = network_error.into();
        assert!(matches!(fetch_error, FetchError::Network(_)));

        let abort_error = AbortError::new("test");
        let fetch_error: FetchError = abort_error.into();
        assert!(matches!(fetch_error, FetchError::Abort(_)));
    }

    #[test]
    fn test_error_messages() {
        let type_error = TypeError::new("invalid input");
        assert_eq!(type_error.message(), "invalid input");

        let network_error = NetworkError::new("timeout");
        assert_eq!(network_error.message(), "timeout");

        let abort_error = AbortError::new("cancelled");
        assert_eq!(abort_error.message(), "cancelled");
    }
}
