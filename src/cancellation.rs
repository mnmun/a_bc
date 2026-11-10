//! # Cloneable thread-safe cancellation flag
//!
//! ![](https://github.com/mnmun/images/blob/main/stop.png?raw=true)
//!
//! Defines a cloneable thread-safe cancellation [`flag`].
//!
//! ---
//!
//! See the [`crate`] documentation for usage example.
//!
//! [`flag`]: crate::Flag

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use getset::Getters;

/// # Cloneable thread-safe cancellation `flag`
///
/// ![](https://github.com/mnmun/images/blob/main/stop.png?raw=true)
///
/// Cancellation `flag` that can be safely cloned and transferred between
/// multiple threads.
///
/// ## Creation
///
/// Can be created using the [`Flag::new()`] method:
///
/// ```rust
/// use a_bc::Flag;
///
/// let flag = Flag::new();
/// ```
///
/// ## Example
///
/// Cancellation `flag` can be put into the cancelled state using the
/// [`cancel()`] method:
///
/// ```rust
/// use std::thread;
/// use a_bc::Flag;
///
/// let flag = Flag::new();
/// assert!(!flag.is_cancelled());
///
/// let handler = thread::spawn({
///     let flag = flag.clone();
///
///     move || {
///         while !flag.is_cancelled() {}
///         true
///     }
/// });
///
/// flag.cancel();
/// assert!(flag.is_cancelled());
///
/// assert!(handler.join().unwrap());
/// ```
///
/// ---
///
/// See the [`module`] documentation for more information.
///
/// [`Flag::new()`]: crate::Flag::new()
/// [`cancel()`]: crate::Flag::cancel()
/// [`module`]: crate::cancellation
#[repr(transparent)]
#[derive(Clone, Debug, Getters, Default)]
pub struct Flag(Arc<AtomicBool>);

impl Flag {
    /// # Creates a new cancellation [`flag`] in the non-cancelled state
    ///
    /// See the [`flag`] documentation for more information.
    ///
    /// [`flag`]: crate::Flag
    pub fn new() -> Self {
        Self::default()
    }

    /// # Returns the cancellation [`flag`] status
    ///
    /// See the [`flag`] documentation for more information.
    ///
    /// [`flag`]: crate::Flag
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }

    /// # Requests cancellation
    ///
    /// See the [`flag`] documentation for more information.
    ///
    /// [`flag`]: crate::Flag
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
}

impl PartialEq for Flag {
    fn eq(&self, other: &Self) -> bool {
        self.is_cancelled() == other.is_cancelled()
    }
}

impl Eq for Flag {}
