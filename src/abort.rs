//! Request cancellation support using abort signals and controllers.
//!
//! This module provides the [`AbortSignal`] and [`AbortController`] types for
//! cancelling HTTP requests. This follows the WHATWG Fetch specification and
//! DOM AbortController standard.
//!
//! # Usage Examples
//!
//! ## Basic Abort Controller
//!
//! ```rust
//! use fetchttp::*;
//! use std::time::Duration;
//! use tokio::time::sleep;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let controller = AbortController::new();
//!     let signal = controller.signal().clone();
//!     
//!     let mut init = RequestInit::new();
//!     init.signal = Some(signal);
//!     
//!     // Cancel the request after 1 second
//!     tokio::spawn(async move {
//!         sleep(Duration::from_secs(1)).await;
//!         controller.abort();
//!     });
//!     
//!     match fetch("https://httpbin.org/delay/5", Some(init)).await {
//!         Ok(_) => println!("Request completed"),
//!         Err(FetchError::Abort(_)) => println!("Request was cancelled"),
//!         Err(e) => println!("Request failed: {}", e),
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Pre-aborted Signal
//!
//! ```rust
//! use fetchttp::AbortSignal;
//!
//! // Create a signal that is already aborted
//! let signal = AbortSignal::abort(Some("Operation cancelled".to_string()));
//! assert!(signal.aborted());
//! assert_eq!(signal.reason().unwrap(), "Operation cancelled");
//! ```

use std::sync::{Arc, Mutex};

/// A signal that can be used to cancel operations.
///
/// `AbortSignal` represents the signal side of the abort functionality. It can
/// be checked to see if an operation should be cancelled and can provide a
/// reason for the cancellation.
///
/// Signals can be created in two ways:
/// 1. Through an [`AbortController`] for dynamic cancellation
/// 2. Using [`AbortSignal::abort()`] for pre-cancelled signals
///
/// # Thread Safety
///
/// `AbortSignal` is thread-safe and can be safely shared between threads.
/// Multiple clones of the same signal will all reflect the same abort state.
///
/// # Examples
///
/// ```rust
/// use fetchttp::{AbortController, AbortSignal};
///
/// // Create through controller
/// let controller = AbortController::new();
/// let signal = controller.signal().clone();
/// assert!(!signal.aborted());
///
/// controller.abort();
/// assert!(signal.aborted());
///
/// // Create pre-aborted
/// let aborted_signal = AbortSignal::abort(Some("Cancelled".to_string()));
/// assert!(aborted_signal.aborted());
/// assert_eq!(aborted_signal.reason().unwrap(), "Cancelled");
/// ```
#[derive(Debug, Clone)]
pub struct AbortSignal {
    /// Shared state between signal clones
    inner: Arc<Mutex<AbortSignalInner>>,
}

/// Internal state of an abort signal.
///
/// This struct holds the mutable state that is shared between all clones
/// of an `AbortSignal`.
#[derive(Debug)]
struct AbortSignalInner {
    /// Whether the signal has been aborted
    aborted: bool,
    /// Optional reason for the abort
    reason: Option<String>,
}

impl AbortSignal {
    /// Create a new abort signal that is not aborted.
    ///
    /// This signal can later be aborted through an [`AbortController`] or
    /// by calling [`do_abort()`] internally.
    ///
    /// [`do_abort()`]: AbortSignal::do_abort
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetchttp::AbortSignal;
    ///
    /// let signal = AbortSignal::new();
    /// assert!(!signal.aborted());
    /// assert!(signal.reason().is_none());
    /// ```
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(AbortSignalInner {
                aborted: false,
                reason: None,
            })),
        }
    }

    /// Create an abort signal that is already aborted.
    ///
    /// This is useful for creating signals that represent operations that
    /// should be cancelled immediately.
    ///
    /// # Arguments
    ///
    /// * `reason` - Optional reason for the abort
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetchttp::AbortSignal;
    ///
    /// let signal = AbortSignal::abort(Some("Timeout".to_string()));
    /// assert!(signal.aborted());
    /// assert_eq!(signal.reason().unwrap(), "Timeout");
    ///
    /// let signal_no_reason = AbortSignal::abort(None);
    /// assert!(signal_no_reason.aborted());
    /// assert!(signal_no_reason.reason().is_none());
    /// ```
    pub fn abort(reason: Option<String>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(AbortSignalInner {
                aborted: true,
                reason,
            })),
        }
    }

    /// Check if the signal has been aborted.
    ///
    /// This method is typically called by operations that support cancellation
    /// to check if they should stop processing.
    ///
    /// # Returns
    ///
    /// `true` if the signal has been aborted, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetchttp::AbortController;
    ///
    /// let controller = AbortController::new();
    /// let signal = controller.signal();
    ///
    /// assert!(!signal.aborted());
    ///
    /// controller.abort();
    /// assert!(signal.aborted());
    /// ```
    pub fn aborted(&self) -> bool {
        self.inner.lock().unwrap().aborted
    }

    /// Get the reason for the abort, if any.
    ///
    /// Returns the reason string provided when the signal was aborted, or
    /// `None` if no reason was provided or the signal hasn't been aborted.
    ///
    /// # Returns
    ///
    /// `Some(String)` with the abort reason, or `None` if no reason was set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetchttp::AbortSignal;
    ///
    /// let signal = AbortSignal::abort(Some("User cancelled".to_string()));
    /// assert_eq!(signal.reason().unwrap(), "User cancelled");
    ///
    /// let signal_no_reason = AbortSignal::abort(None);
    /// assert!(signal_no_reason.reason().is_none());
    /// ```
    pub fn reason(&self) -> Option<String> {
        self.inner.lock().unwrap().reason.clone()
    }

    /// Internal method to abort the signal.
    ///
    /// This method is used internally by [`AbortController`] to abort the signal.
    /// Once a signal is aborted, it cannot be un-aborted. Subsequent calls to
    /// this method will not change the reason.
    ///
    /// # Arguments
    ///
    /// * `reason` - Optional reason for the abort
    pub(crate) fn do_abort(&self, reason: Option<String>) {
        let mut inner = self.inner.lock().unwrap();
        if !inner.aborted {
            inner.aborted = true;
            inner.reason = reason;
        }
    }
}

impl Default for AbortSignal {
    fn default() -> Self {
        Self::new()
    }
}

/// A controller for managing abort signals.
///
/// `AbortController` provides a way to create and control [`AbortSignal`]s.
/// It follows the DOM AbortController specification and allows for cancelling
/// operations by calling the [`abort()`] method.
///
/// [`abort()`]: AbortController::abort
///
/// Each controller manages exactly one signal, which can be retrieved using
/// the [`signal()`] method. The signal can be cloned and shared as needed.
///
/// [`signal()`]: AbortController::signal
///
/// # Examples
///
/// ```rust
/// use fetchttp::AbortController;
///
/// let controller = AbortController::new();
/// let signal = controller.signal().clone();
///
/// // Use the signal in an operation
/// assert!(!signal.aborted());
///
/// // Cancel the operation
/// controller.abort();
/// assert!(signal.aborted());
/// ```
#[derive(Debug)]
pub struct AbortController {
    /// The signal managed by this controller
    signal: AbortSignal,
}

impl AbortController {
    /// Create a new abort controller with a new signal.
    ///
    /// The signal starts in a non-aborted state and can be aborted later
    /// by calling [`abort()`].
    ///
    /// [`abort()`]: AbortController::abort
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetchttp::AbortController;
    ///
    /// let controller = AbortController::new();
    /// assert!(!controller.signal().aborted());
    /// ```
    pub fn new() -> Self {
        Self {
            signal: AbortSignal::new(),
        }
    }

    /// Get a reference to the signal managed by this controller.
    ///
    /// The signal can be cloned and shared as needed. All clones will
    /// reflect the same abort state.
    ///
    /// # Returns
    ///
    /// A reference to the [`AbortSignal`] managed by this controller.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetchttp::AbortController;
    ///
    /// let controller = AbortController::new();
    /// let signal1 = controller.signal().clone();
    /// let signal2 = controller.signal().clone();
    ///
    /// // Both signals will show the same state
    /// controller.abort();
    /// assert!(signal1.aborted());
    /// assert!(signal2.aborted());
    /// ```
    pub fn signal(&self) -> &AbortSignal {
        &self.signal
    }

    /// Abort the signal managed by this controller.
    ///
    /// This method aborts the signal with a default reason of "AbortError".
    /// Once aborted, the signal cannot be un-aborted.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fetchttp::AbortController;
    ///
    /// let controller = AbortController::new();
    /// let signal = controller.signal();
    ///
    /// assert!(!signal.aborted());
    ///
    /// controller.abort();
    /// assert!(signal.aborted());
    /// assert_eq!(signal.reason().unwrap(), "AbortError");
    /// ```
    pub fn abort(&self) {
        self.signal.do_abort(Some("AbortError".to_string()));
    }
}

impl Default for AbortController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abort_signal_new() {
        let signal = AbortSignal::new();
        assert!(!signal.aborted());
        assert!(signal.reason().is_none());
    }

    #[test]
    fn test_abort_signal_abort() {
        let signal = AbortSignal::abort(Some("Test reason".to_string()));
        assert!(signal.aborted());
        assert_eq!(signal.reason().unwrap(), "Test reason");
    }

    #[test]
    fn test_abort_signal_abort_no_reason() {
        let signal = AbortSignal::abort(None);
        assert!(signal.aborted());
        assert!(signal.reason().is_none());
    }

    #[test]
    fn test_abort_signal_default() {
        let signal = AbortSignal::default();
        assert!(!signal.aborted());
        assert!(signal.reason().is_none());
    }

    #[test]
    fn test_abort_controller_new() {
        let controller = AbortController::new();
        assert!(!controller.signal().aborted());
    }

    #[test]
    fn test_abort_controller_abort() {
        let controller = AbortController::new();
        assert!(!controller.signal().aborted());

        controller.abort();
        assert!(controller.signal().aborted());
        assert_eq!(controller.signal().reason().unwrap(), "AbortError");
    }

    #[test]
    fn test_abort_controller_default() {
        let controller = AbortController::default();
        assert!(!controller.signal().aborted());
    }

    #[test]
    fn test_abort_signal_do_abort() {
        let signal = AbortSignal::new();
        assert!(!signal.aborted());

        signal.do_abort(Some("Manual abort".to_string()));
        assert!(signal.aborted());
        assert_eq!(signal.reason().unwrap(), "Manual abort");

        // Second abort should not change reason
        signal.do_abort(Some("Second abort".to_string()));
        assert_eq!(signal.reason().unwrap(), "Manual abort");
    }

    #[test]
    fn test_abort_signal_clone() {
        let signal = AbortSignal::new();
        let cloned = signal.clone();

        // Initially both should not be aborted
        assert!(!signal.aborted());
        assert!(!cloned.aborted());

        signal.do_abort(Some("Test".to_string()));

        // Both should show aborted since they share the same inner state
        assert!(signal.aborted());
        assert!(cloned.aborted());
        assert_eq!(signal.reason().unwrap(), "Test");
        assert_eq!(cloned.reason().unwrap(), "Test");
    }

    #[test]
    fn test_signal_sharing_between_threads() {
        use std::thread;
        use std::time::Duration;

        let controller = AbortController::new();
        let signal = controller.signal().clone();

        let handle = thread::spawn(move || {
            thread::sleep(Duration::from_millis(10));
            controller.abort();
        });

        // Wait for the other thread to abort
        handle.join().unwrap();

        // Signal should now be aborted
        assert!(signal.aborted());
    }

    #[test]
    fn test_multiple_signal_clones() {
        let controller = AbortController::new();
        let signal1 = controller.signal().clone();
        let signal2 = controller.signal().clone();
        let signal3 = controller.signal().clone();

        // All should start not aborted
        assert!(!signal1.aborted());
        assert!(!signal2.aborted());
        assert!(!signal3.aborted());

        controller.abort();

        // All should now be aborted
        assert!(signal1.aborted());
        assert!(signal2.aborted());
        assert!(signal3.aborted());

        // All should have the same reason
        assert_eq!(signal1.reason().unwrap(), "AbortError");
        assert_eq!(signal2.reason().unwrap(), "AbortError");
        assert_eq!(signal3.reason().unwrap(), "AbortError");
    }
}
