//! HTTP headers management following the WHATWG Fetch specification.
//!
//! This module provides the [`Headers`] type for managing HTTP headers in a way
//! that closely follows the web standard. Headers are case-insensitive and support
//! all standard header operations.

use crate::error::{FetchError, Result, TypeError};
use std::collections::HashMap;

/// HTTP headers container following the WHATWG Fetch specification.
///
/// The `Headers` type provides a case-insensitive map for HTTP headers with
/// validation according to the web standards. Header names are normalized to
/// lowercase for consistent access.
///
/// # Examples
///
/// ```rust
/// use fetch::Headers;
///
/// let mut headers = Headers::new();
/// headers.set("Content-Type", "application/json").unwrap();
/// headers.set("Accept", "application/json").unwrap();
///
/// assert!(headers.has("content-type").unwrap());
/// assert_eq!(headers.get("Content-Type").unwrap().unwrap(), "application/json");
///
/// // Append to existing header
/// headers.append("Accept", "text/plain").unwrap();
/// assert_eq!(headers.get("accept").unwrap().unwrap(), "application/json, text/plain");
/// ```
#[derive(Debug, Clone, Default)]
pub struct Headers {
    /// Internal map storing header name-value pairs.
    /// Names are stored in lowercase for case-insensitive access.
    map: HashMap<String, String>,
}

impl Headers {
    /// Create a new empty Headers instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetch::Headers;
    ///
    /// let headers = Headers::new();
    /// assert_eq!(headers.keys().count(), 0);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a value to an existing header or create a new one.
    ///
    /// If the header already exists, the new value is appended with a comma
    /// separator. If it doesn't exist, a new header is created.
    ///
    /// # Arguments
    ///
    /// * `name` - The header name (case-insensitive)
    /// * `value` - The header value to append
    ///
    /// # Errors
    ///
    /// Returns a [`TypeError`] if the header name or value is invalid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetch::Headers;
    ///
    /// let mut headers = Headers::new();
    /// headers.set("Accept", "application/json").unwrap();
    /// headers.append("Accept", "text/plain").unwrap();
    ///
    /// assert_eq!(headers.get("Accept").unwrap().unwrap(), "application/json, text/plain");
    /// ```
    pub fn append(&mut self, name: &str, value: &str) -> Result<()> {
        let name = self.validate_name(name)?;
        let value = self.validate_value(value)?;

        match self.map.get(&name) {
            Some(existing) => {
                self.map.insert(name, format!("{}, {}", existing, value));
            }
            None => {
                self.map.insert(name, value);
            }
        }
        Ok(())
    }

    /// Delete a header.
    ///
    /// Removes the header with the given name. If the header doesn't exist,
    /// this operation is a no-op.
    ///
    /// # Arguments
    ///
    /// * `name` - The header name to delete (case-insensitive)
    ///
    /// # Errors
    ///
    /// Returns a [`TypeError`] if the header name is invalid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetch::Headers;
    ///
    /// let mut headers = Headers::new();
    /// headers.set("Content-Type", "application/json").unwrap();
    /// assert!(headers.has("Content-Type").unwrap());
    ///
    /// headers.delete("Content-Type").unwrap();
    /// assert!(!headers.has("Content-Type").unwrap());
    /// ```
    pub fn delete(&mut self, name: &str) -> Result<()> {
        let name = self.validate_name(name)?;
        self.map.remove(&name);
        Ok(())
    }

    /// Get the value of a header.
    ///
    /// Returns the value of the header with the given name, or `None` if the
    /// header doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `name` - The header name to get (case-insensitive)
    ///
    /// # Returns
    ///
    /// `Some(String)` with the header value, or `None` if not found.
    ///
    /// # Errors
    ///
    /// Returns a [`TypeError`] if the header name is invalid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetch::Headers;
    ///
    /// let mut headers = Headers::new();
    /// headers.set("Content-Type", "application/json").unwrap();
    ///
    /// assert_eq!(headers.get("Content-Type").unwrap().unwrap(), "application/json");
    /// assert_eq!(headers.get("content-type").unwrap().unwrap(), "application/json");
    /// assert!(headers.get("Accept").unwrap().is_none());
    /// ```
    pub fn get(&self, name: &str) -> Result<Option<String>> {
        let name = self.validate_name(name)?;
        Ok(self.map.get(&name).cloned())
    }

    /// Get all Set-Cookie header values.
    ///
    /// The Set-Cookie header is special because it can have multiple values
    /// that shouldn't be combined with commas. This method returns all values
    /// as separate strings.
    ///
    /// # Returns
    ///
    /// A vector of Set-Cookie header values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetch::Headers;
    ///
    /// let mut headers = Headers::new();
    /// headers.set("Set-Cookie", "session=abc123, secure=true").unwrap();
    ///
    /// let cookies = headers.get_set_cookie();
    /// assert_eq!(cookies.len(), 2);
    /// ```
    pub fn get_set_cookie(&self) -> Vec<String> {
        self.map
            .get("set-cookie")
            .map(|v| v.split(", ").map(|s| s.to_string()).collect())
            .unwrap_or_default()
    }

    /// Check if a header exists.
    ///
    /// Returns `true` if a header with the given name exists, `false` otherwise.
    ///
    /// # Arguments
    ///
    /// * `name` - The header name to check (case-insensitive)
    ///
    /// # Errors
    ///
    /// Returns a [`TypeError`] if the header name is invalid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetch::Headers;
    ///
    /// let mut headers = Headers::new();
    /// headers.set("Content-Type", "application/json").unwrap();
    ///
    /// assert!(headers.has("Content-Type").unwrap());
    /// assert!(headers.has("content-type").unwrap());
    /// assert!(!headers.has("Accept").unwrap());
    /// ```
    pub fn has(&self, name: &str) -> Result<bool> {
        let name = self.validate_name(name)?;
        Ok(self.map.contains_key(&name))
    }

    /// Set a header value.
    ///
    /// Sets the header to the given value, replacing any existing value.
    ///
    /// # Arguments
    ///
    /// * `name` - The header name (case-insensitive)
    /// * `value` - The header value
    ///
    /// # Errors
    ///
    /// Returns a [`TypeError`] if the header name or value is invalid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetch::Headers;
    ///
    /// let mut headers = Headers::new();
    /// headers.set("Content-Type", "application/json").unwrap();
    /// headers.set("Accept", "application/json").unwrap();
    ///
    /// assert_eq!(headers.get("Content-Type").unwrap().unwrap(), "application/json");
    /// ```
    pub fn set(&mut self, name: &str, value: &str) -> Result<()> {
        let name = self.validate_name(name)?;
        let value = self.validate_value(value)?;
        self.map.insert(name, value);
        Ok(())
    }

    /// Iterate over all header name-value pairs.
    ///
    /// Returns an iterator that yields tuples of (name, value) for all headers.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetch::Headers;
    ///
    /// let mut headers = Headers::new();
    /// headers.set("Content-Type", "application/json").unwrap();
    /// headers.set("Accept", "application/json").unwrap();
    ///
    /// for (name, value) in headers.entries() {
    ///     println!("{}: {}", name, value);
    /// }
    /// ```
    pub fn entries(&self) -> impl Iterator<Item = (&str, &str)> {
        self.map.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    /// Iterate over all header names.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetch::Headers;
    ///
    /// let mut headers = Headers::new();
    /// headers.set("Content-Type", "application/json").unwrap();
    /// headers.set("Accept", "application/json").unwrap();
    ///
    /// let names: Vec<_> = headers.keys().collect();
    /// assert_eq!(names.len(), 2);
    /// ```
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.map.keys().map(|k| k.as_str())
    }

    /// Iterate over all header values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetch::Headers;
    ///
    /// let mut headers = Headers::new();
    /// headers.set("Content-Type", "application/json").unwrap();
    /// headers.set("Accept", "application/json").unwrap();
    ///
    /// let values: Vec<_> = headers.values().collect();
    /// assert_eq!(values.len(), 2);
    /// ```
    pub fn values(&self) -> impl Iterator<Item = &str> {
        self.map.values().map(|v| v.as_str())
    }

    /// Validate a header name according to HTTP standards.
    ///
    /// Header names must be valid HTTP tokens and are normalized to lowercase.
    fn validate_name(&self, name: &str) -> Result<String> {
        if name.is_empty() {
            return Err(FetchError::Type(TypeError::new("Invalid header name")));
        }

        // HTTP token characters: VCHAR except delimiters
        for byte in name.bytes() {
            if !matches!(byte, b'!' | b'#'..=b'\'' | b'*' | b'+' | b'-' | b'.' | b'0'..=b'9' | b'A'..=b'Z' | b'^'..=b'z' | b'|' | b'~')
            {
                return Err(FetchError::Type(TypeError::new("Invalid header name")));
            }
        }

        Ok(name.to_ascii_lowercase())
    }

    /// Validate a header value according to HTTP standards.
    ///
    /// Header values are trimmed of leading/trailing whitespace and validated
    /// for allowed characters.
    fn validate_value(&self, value: &str) -> Result<String> {
        let trimmed = value.trim_matches(|c| c == ' ' || c == '\t');

        // HTTP field value characters: VCHAR, WSP
        for byte in trimmed.bytes() {
            if !matches!(byte, 0x21..=0x7E | b' ' | b'\t') {
                return Err(FetchError::Type(TypeError::new("Invalid header value")));
            }
        }

        Ok(trimmed.to_string())
    }

    /// Convert to hyper's HeaderMap for internal use.
    ///
    /// This method is used internally to convert our Headers type to hyper's
    /// HeaderMap for HTTP requests.
    pub(crate) fn to_http_headers(&self) -> Result<http::HeaderMap> {
        let mut map = http::HeaderMap::new();
        for (name, value) in &self.map {
            let header_name = http::header::HeaderName::from_bytes(name.as_bytes())
                .map_err(|_| FetchError::Type(TypeError::new("Invalid header name")))?;
            let header_value = http::header::HeaderValue::from_str(value)
                .map_err(|_| FetchError::Type(TypeError::new("Invalid header value")))?;
            map.insert(header_name, header_value);
        }
        Ok(map)
    }

    /// Create Headers from hyper's HeaderMap.
    ///
    /// This method is used internally to convert hyper's HeaderMap to our
    /// Headers type for HTTP responses.
    pub(crate) fn from_http_headers(headers: &http::HeaderMap) -> Self {
        let mut map = HashMap::new();
        for (name, value) in headers {
            if let Ok(value_str) = value.to_str() {
                map.insert(name.as_str().to_ascii_lowercase(), value_str.to_string());
            }
        }
        Self { map }
    }
}

// Convenient conversion from arrays
impl<const N: usize> From<&[(&str, &str); N]> for Headers {
    fn from(headers: &[(&str, &str); N]) -> Self {
        let mut h = Self::new();
        for (name, value) in headers {
            let _ = h.set(name, value);
        }
        h
    }
}

impl From<&[(&str, &str)]> for Headers {
    fn from(headers: &[(&str, &str)]) -> Self {
        let mut h = Self::new();
        for (name, value) in headers {
            let _ = h.set(name, value);
        }
        h
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_headers_basic_operations() {
        let mut headers = Headers::new();

        // Test set/get
        headers.set("content-type", "application/json").unwrap();
        assert_eq!(
            headers.get("content-type").unwrap().unwrap(),
            "application/json"
        );

        // Test case insensitive
        assert_eq!(
            headers.get("Content-Type").unwrap().unwrap(),
            "application/json"
        );

        // Test has
        assert!(headers.has("content-type").unwrap());
        assert!(!headers.has("x-nonexistent").unwrap());

        // Test append
        headers.append("accept", "application/json").unwrap();
        headers.append("accept", "text/plain").unwrap();
        assert_eq!(
            headers.get("accept").unwrap().unwrap(),
            "application/json, text/plain"
        );

        // Test delete
        headers.delete("content-type").unwrap();
        assert!(!headers.has("content-type").unwrap());
    }

    #[test]
    fn test_headers_validation() {
        let mut headers = Headers::new();

        // Invalid header name (empty)
        assert!(headers.set("", "value").is_err());

        // Invalid header name (control character)
        assert!(headers.set("test\x00", "value").is_err());

        // Invalid header value (control character)
        assert!(headers.set("test", "value\r\n").is_err());

        // Valid headers
        assert!(headers.set("x-custom", "value").is_ok());
        assert!(headers.set("content-type", "application/json").is_ok());
    }

    #[test]
    fn test_headers_iteration() {
        let mut headers = Headers::new();
        headers.set("a", "1").unwrap();
        headers.set("b", "2").unwrap();
        headers.set("c", "3").unwrap();

        let entries: Vec<_> = headers.entries().collect();
        assert_eq!(entries.len(), 3);

        let keys: Vec<_> = headers.keys().collect();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&"a"));
        assert!(keys.contains(&"b"));
        assert!(keys.contains(&"c"));

        let values: Vec<_> = headers.values().collect();
        assert_eq!(values.len(), 3);
        assert!(values.contains(&"1"));
        assert!(values.contains(&"2"));
        assert!(values.contains(&"3"));
    }

    #[test]
    fn test_headers_from_slice() {
        let headers = Headers::from(
            &[
                ("content-type", "application/json"),
                ("accept", "application/json"),
            ][..],
        );

        assert_eq!(
            headers.get("content-type").unwrap().unwrap(),
            "application/json"
        );
        assert_eq!(headers.get("accept").unwrap().unwrap(), "application/json");
    }

    #[test]
    fn test_headers_from_array() {
        let headers = Headers::from(&[
            ("content-type", "application/json"),
            ("accept", "application/json"),
        ]);

        assert_eq!(
            headers.get("content-type").unwrap().unwrap(),
            "application/json"
        );
        assert_eq!(headers.get("accept").unwrap().unwrap(), "application/json");
    }

    #[test]
    fn test_get_set_cookie() {
        let mut headers = Headers::new();
        headers
            .set("set-cookie", "session=abc123, secure=true")
            .unwrap();

        let cookies = headers.get_set_cookie();
        assert_eq!(cookies.len(), 2);
        assert!(cookies.contains(&"session=abc123".to_string()));
        assert!(cookies.contains(&"secure=true".to_string()));
    }
}
