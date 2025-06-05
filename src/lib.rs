//! # fetchttp
//!
//! A WHATWG Fetch API compliant HTTP client library for Rust.
//!
//! This library provides a familiar `fetch()` API that closely follows the
//! [WHATWG Fetch specification](https://fetch.spec.whatwg.org/), making it easy
//! for developers familiar with web APIs to use in Rust applications.
//!
//! ## Features
//!
//! - **Specification Compliant**: Implements the WHATWG Fetch API specification
//! - **Async/Await Support**: Built on Tokio for modern async Rust
//! - **Type Safety**: Leverages Rust's type system for safe HTTP operations
//! - **JSON Support**: Built-in JSON serialization/deserialization with serde
//! - **Flexible Bodies**: Support for text, bytes, and JSON request/response bodies
//! - **Header Management**: Complete header manipulation API
//! - **Request/Response Cloning**: Efficient cloning following the specification
//! - **Abort Signals**: Request cancellation support
//!
//! ## Quick Start
//!
//! ### Simple GET Request
//!
//! ```rust
//! use fetchttp::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let response = fetch("https://api.github.com/users/octocat", None).await?;
//!     
//!     if response.ok() {
//!         let user: serde_json::Value = response.json().await?;
//!         println!("User: {}", user["name"]);
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! ### POST Request with JSON
//!
//! ```rust
//! use fetchttp::*;
//! use serde_json::json;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let data = json!({
//!         "name": "John Doe",
//!         "email": "john@example.com"
//!     });
//!     
//!     let mut init = RequestInit::new();
//!     init.method = Some("POST".to_string());
//!     init.body = Some(ReadableStream::from_json(&data));
//!     
//!     let response = fetch("https://api.example.com/users", Some(init)).await?;
//!     
//!     if response.ok() {
//!         println!("User created successfully!");
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! ### Custom Headers
//!
//! ```rust
//! use fetchttp::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut headers = Headers::new();
//!     headers.set("Authorization", "Bearer your-token")?;
//!     headers.set("User-Agent", "MyApp/1.0")?;
//!     
//!     let mut init = RequestInit::new();
//!     init.headers = Some(headers);
//!     
//!     let response = fetch("https://api.example.com/protected", Some(init)).await?;
//!     let text = response.text().await?;
//!     
//!     println!("Response: {}", text);
//!     Ok(())
//! }
//! ```
//!
//! ## API Overview
//!
//! The main entry point is the [`fetch`] function, which takes a URL and optional
//! [`RequestInit`] configuration. The function returns a [`Response`] that can be
//! consumed in various ways:
//!
//! - [`Response::text()`] - Get response as text
//! - [`Response::json()`] - Parse response as JSON
//! - [`Response::array_buffer()`] - Get response as bytes
//! - [`Response::blob()`] - Get response as blob (bytes)
//!
//! ## Error Handling
//!
//! The library uses a comprehensive error system with specific error types:
//!
//! - [`TypeError`] - Invalid arguments or operations
//! - [`NetworkError`] - Network-related failures
//! - [`AbortError`] - Request was aborted
//!
//! All errors implement the standard Rust error traits.

mod abort;
mod body;
mod client;
mod error;
mod headers;
mod request;
mod response;

// Re-export all public types and functions
pub use abort::{AbortController, AbortSignal};
pub use body::ReadableStream;
pub use client::fetch;
pub use error::{AbortError, FetchError, NetworkError, Result, TypeError};
pub use headers::Headers;
pub use request::{
    Request, RequestCache, RequestCredentials, RequestInit, RequestMode, RequestRedirect,
};
pub use response::{Response, ResponseInit, ResponseType};

// Re-export commonly used external types
pub use bytes::Bytes;
pub use serde_json::{Map as JsonMap, Value as JsonValue};
