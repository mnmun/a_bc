//! # A cloneable thread-safe cancellation flag
//!
//! ![](https://github.com/mnmun/images/blob/main/stop.png?raw=true)
//!
//! Provides [`cancel`] - a cloneable thread-safe cancellation flag.
//!
//! ---
//!
//! See the [`crate documentation`] for usage example.
//!
//! [`cancel`]: Cancel
//! [`crate documentation`]: crate

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use getset::Getters;

/// # A cloneable thread-safe cancellation flag
///
/// ![](https://github.com/mnmun/images/blob/main/stop.png?raw=true)
///
/// Represents a cancellation flag that can be safely cloned and transferred
/// between multiple threads.
///
/// The flag is managed and inspected through the following methods:
///
/// - [`cancel()`] - requests the cancellation;
/// - [`is_cancelled()`] - returns `true` if cancellation has been
///   [`requested`], and `false` otherwise.
///
/// # Example
///
/// ```
/// use a_bc::Cancel;
///
/// let flag = Cancel::new();
/// assert!(!flag.is_cancelled());
///
/// flag.cancel();
/// assert!(flag.is_cancelled());
/// ```
///
/// ---
///
/// See the [`module documentation`] for more information.
///
/// [`cancel()`]: Cancel::cancel()
/// [`is_cancelled()`]: Cancel::is_cancelled()
/// [`requested`]: Cancel::cancel()
/// [`module documentation`]: crate::cancel
#[repr(transparent)]
#[derive(Clone, Debug, Getters, Default)]
pub struct Cancel(Arc<AtomicBool>);

impl Cancel {
    /// # Creates a new [`cancellation flag`] in the non-cancelled state (`false`)
    ///
    /// [`cancellation flag`]: Cancel
    pub fn new() -> Self {
        Self::default()
    }

    /// # Returns `true` if cancellation has been [`requested`], and `false` otherwise
    ///
    /// [`requested`]: Cancel::cancel()
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }

    /// # Requests cancellation
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
}

impl PartialEq for Cancel {
    fn eq(&self, other: &Self) -> bool {
        self.is_cancelled() == other.is_cancelled()
    }
}

impl Eq for Cancel {}
