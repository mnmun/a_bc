//! # Cancellation
//!
//! [`Cancel`] is a cloneable, thread-safe cancellation flag. All clones share
//! the same underlying state, so one thread can request cancellation while a
//! lexer on another thread polls it via [`Cancel::is_cancelled()`]. The flag
//! is stored behind an [`Arc`] and toggled with acquire/release memory
//! ordering.

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use getset::Getters;

/// # A thread-safe cancellation flag
///
/// Cloning shares the same underlying flag, allowing cancellation from another
/// thread.
///
/// # Example
///
/// ```
/// use a_bc::cancel::Cancel;
///
/// let cancel = Cancel::new();
/// assert!(!cancel.is_cancelled());
///
/// cancel.cancel();
/// assert!(cancel.is_cancelled());
/// ```
#[derive(Clone, Debug, Getters, Default)]
pub struct Cancel(Arc<AtomicBool>);

impl Cancel {
    /// Creates a new [`Cancel`] flag in the non-cancelled state
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns `true` if cancellation has been requested
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }

    /// Requests cancellation
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
}
