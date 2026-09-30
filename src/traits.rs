//! # [`Token`] traits
//!
//! ![](https://github.com/mnmun/images/blob/main/umbrella.png?raw=true)
//!
//! Provides the following traits:
//!
//! - [`KindBounds`] - a blanket bound that any user-defined token kind type
//!   must satisfy;
//! - [`InnerRange`] - enables a token kind to describe its own inner range, so
//!   delimited tokens can expose their contents without the delimiters.
//!
//! ---
//!
//! See the [`crate documentation`] for more information.
//!
//! [`Token`]: crate::Token
//! [`crate documentation`]: crate

use std::{
    fmt::{Debug, Display},
    ops::Range,
};

/// # Bound on the `Kind` type parameter
///
/// Requires [`PartialEq`] + [`Clone`] + [`Copy`] + [`Debug`] + [`Display`].
///
/// A blanket implementation covers all types that satisfy these bounds, so no
/// manual implementation is needed.
///
/// ---
///
/// See the [`module documentation`] for more information.
pub trait KindBounds: PartialEq + Clone + Copy + Debug + Display {}
impl<T> KindBounds for T where T: PartialEq + Clone + Copy + Debug + Display {}

/// # Provides the inner `range` of a delimited `token`
///
/// For delimited tokens like `{...}`, `[...]` or `"..."`, this returns the
/// `range` without the delimiters.
///
/// Returns `None` for tokens that have no inner content.
///
/// ---
///
/// See the [`module documentation`] for more information.
pub trait InnerRange {
    /// # Returns the token inner range if it exists, otherwise `None`
    fn inner_range(&self, range: &Range<usize>) -> Option<Range<usize>>;
}
