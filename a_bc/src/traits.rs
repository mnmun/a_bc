//! # Traits for user-defined token kinds
//!
//! - [`KindBounds`] - a blanket bound that any user token-kind type (e.g. an
//!   `enum`) should satisfy
//! - [`InnerRange`] - lets a token kind describe its own inner range, so
//!   delimited tokens can expose their contents without the delimiters
//!
//! See [`crate`] for more information.

use std::{fmt::Debug, ops::Range};

/// # Bound on the `Kind` type parameter
///
/// Requires [`PartialEq`] + [`Clone`] + [`Copy`] + [`Debug`]. A blanket impl
/// covers all types that satisfy these bounds, so no manual implementation is
/// needed.
pub trait KindBounds: PartialEq + Clone + Copy + Debug {}
impl<T> KindBounds for T where T: PartialEq + Clone + Copy + Debug {}

/// # Provides the inner `range` of a delimited `token`
///
/// For bracketed tokens like `{...}`, `[...]` or `"..."`, this returns the
/// range without the delimiters. Returns `None` for tokens that have no inner
/// content (e.g. punctuation).
pub trait InnerRange {
    /// Returns the token inner range if it exists, otherwise `None`
    fn inner_range(&self, range: &Range<usize>) -> Option<Range<usize>>;
}
