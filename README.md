# fetchttp

[![Crates.io](https://img.shields.io/crates/v/fetchttp)](https://crates.io/crates/fetchttp)
[![Documentation](https://docs.rs/fetchttp/badge.svg)](https://docs.rs/fetchttp)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![CI](https://github.com/MuntasirSZN/fetchttp/actions/workflows/test.yml/badge.svg)](https://github.com/MuntasirSZN/fetchttp/actions/workflows/test.yml)

A **WHATWG Fetch API** compliant HTTP client library for Rust that provides the familiar `fetch()` interface you know and love from web development.

## ✨ Features

- 🌐 **WHATWG Fetch API Compliant** - Follows the official specification
- 🚀 **Async/Await Support** - Built on Tokio for modern async Rust  
- 🔒 **Type Safety** - Leverages Rust's type system for safe HTTP operations
- 📦 **JSON Support** - Built-in JSON serialization/deserialization with serde
- 🔧 **Flexible Bodies** - Support for text, bytes, and JSON request/response bodies
- 📋 **Header Management** - Complete header manipulation API
- 🔄 **Request/Response Cloning** - Efficient cloning following the specification
- ⏹️ **Abort Signals** - Request cancellation support
- 🔗 **Connection Pooling** - Automatic connection reuse for better performance

## 🚀 Quick Start

Add `fetchttp` to your `Cargo.toml`:

```toml
[dependencies]
fetchttp = "1.0.0"
tokio = { version = "1.0", features = ["full"] }
serde_json = "1.0"  # For JSON support
```

### Simple GET Request

```rust
use fetchttp::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let response = fetch("https://api.github.com/users/octocat", None).await?;
    
    if response.ok() {
        let user: serde_json::Value = response.json().await?;
        println!("User: {}", user["name"]);
    }
    
    Ok(())
}
```

### POST Request with JSON

```rust
use fetchttp::*;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = json!({
        "name": "John Doe",
        "email": "john@example.com"
    });
    
    let mut init = RequestInit::new();
    init.method = Some("POST".to_string());
    init.body = Some(ReadableStream::from_json(&data));
    
    let response = fetch("https://api.example.com/users", Some(init)).await?;
    
    if response.ok() {
        println!("User created successfully!");
    }
    
    Ok(())
}
```

### Custom Headers

```rust
use fetchttp::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut headers = Headers::new();
    headers.set("Authorization", "Bearer your-token")?;
    headers.set("User-Agent", "MyApp/1.0")?;
    
    let mut init = RequestInit::new();
    init.headers = Some(headers);
    
    let response = fetch("https://api.example.com/protected", Some(init)).await?;
    let text = response.text().await?;
    
    println!("Response: {}", text);
    Ok(())
}
```

### Request Cancellation

```rust
use fetchttp::*;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let controller = AbortController::new();
    let signal = controller.signal().clone();
    
    let mut init = RequestInit::new();
    init.signal = Some(signal);
    
    // Cancel the request after 1 second
    tokio::spawn(async move {
        sleep(Duration::from_secs(1)).await;
        controller.abort();
    });
    
    match fetch("https://httpbin.org/delay/5", Some(init)).await {
        Ok(response) => println!("Request completed: {}", response.status()),
        Err(FetchError::Abort(_)) => println!("Request was cancelled"),
        Err(e) => println!("Request failed: {}", e),
    }
    
    Ok(())
}
```

## 📚 API Reference

### Core Functions

- **`fetch(url, init)`** - Perform an HTTP request
  
### Request Types

- **`Request`** - Represents an HTTP request
- **`RequestInit`** - Configuration for requests
- **`RequestMode`** - CORS mode settings
- **`RequestCredentials`** - Credential handling
- **`RequestCache`** - Cache control
- **`RequestRedirect`** - Redirect handling

### Response Types

- **`Response`** - Represents an HTTP response
- **`ResponseInit`** - Configuration for responses  
- **`ResponseType`** - Response type information

### Body and Streams

- **`ReadableStream`** - Request/response body handling
- **`Headers`** - HTTP header management

### Error Handling

- **`FetchError`** - Main error type
- **`TypeError`** - Type validation errors
- **`NetworkError`** - Network-related errors
- **`AbortError`** - Request cancellation errors

### Abort Support

- **`AbortController`** - Controls request cancellation
- **`AbortSignal`** - Signals request cancellation

## 🔄 Body Consumption

Response and request bodies can be consumed in multiple ways:

```rust
// Text
let text = response.text().await?;

// JSON  
let data: MyStruct = response.json().await?;

// Bytes
let bytes = response.array_buffer().await?;

// Blob (alias for array_buffer)
let blob = response.blob().await?;
```

## 🛡️ Error Handling

The library provides comprehensive error handling:

```rust
match fetch("https://example.com", None).await {
    Ok(response) => {
        if response.ok() {
            println!("Success: {}", response.status());
        } else {
            println!("HTTP Error: {}", response.status());
        }
    }
    Err(FetchError::Type(e)) => {
        eprintln!("Type error: {}", e);
    }
    Err(FetchError::Network(e)) => {
        eprintln!("Network error: {}", e);
    }
    Err(FetchError::Abort(e)) => {
        eprintln!("Request aborted: {}", e);
    }
}
```

## 🔧 Advanced Usage

### Custom HTTP Methods

```rust
let mut init = RequestInit::new();
init.method = Some("PATCH".to_string());
init.body = Some(ReadableStream::from_json(&data));

let response = fetch("https://api.example.com/resource/1", Some(init)).await?;
```

### File Upload

```rust
use std::fs;

let file_content = fs::read("document.pdf")?;

let mut init = RequestInit::new();
init.method = Some("POST".to_string());
init.body = Some(ReadableStream::from_bytes(file_content.into()));

let response = fetch("https://api.example.com/upload", Some(init)).await?;
```

### Response Headers

```rust
let response = fetch("https://httpbin.org/response-headers", None).await?;

for (name, value) in response.headers().entries() {
    println!("{}: {}", name, value);
}

// Check specific header
if let Some(content_type) = response.headers().get("content-type")? {
    println!("Content-Type: {}", content_type);
}
```

## 🏗️ Building and Testing

```bash
# Build the library
cargo build

# Run tests
cargo test

# Run benchmarks
cargo bench

# Generate documentation
cargo doc --open
```

## 📊 Performance

The library is designed for performance with:

- Connection pooling via hyper
- Efficient body streaming
- Zero-copy operations where possible
- Minimal allocations

See `benches/` for detailed performance benchmarks.

## 🤝 Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🌟 Comparison with Other Libraries

| Feature | fetchttp | reqwest | hyper | ureq |
|---------|----------|---------|-------|------|
| WHATWG Fetch API | ✅ | ❌ | ❌ | ❌ |
| Async/Await | ✅ | ✅ | ✅ | ❌ |
| JSON Support | ✅ | ✅ | ❌ | ✅ |
| Connection Pooling | ✅ | ✅ | ✅ | ❌ |
| Abort Signals | ✅ | ✅ | ❌ | ❌ |
| Web API Compatibility | ✅ | ❌ | ❌ | ❌ |

## 🔗 Links

- [Documentation](https://docs.rs/fetchttp)
- [Repository](https://github.com/MuntasirSZN/fetchttp)
- [Issues](https://github.com/MuntasirSZN/fetchttp/issues)
- [WHATWG Fetch Specification](https://fetch.spec.whatwg.org/)

---

Made with ❤️ by [MuntasirSZN](https://github.com/MuntasirSZN)
