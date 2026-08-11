//! # Lexed token
//!
//! This module provides [`Token`] for creating custom lexers.
//!
//! See [`crate`] for more information.

use getset::Getters;
use std::ops::Range;

use crate::traits::InnerRange;

/// # Lexed token
///
/// A pair of a user-defined `kind` (typically an `enum`) and the byte `range`
/// the token occupies in the [`Data::source()`].
///
/// [`Data::source()`]: crate::lexer::Data::source()

#[derive(Debug, PartialEq, Clone, Getters)]
#[getset(get = "pub")]
pub struct Token<Kind> {
    /// User-defined token kind (typically an `enum`)
    kind: Kind,

    /// Byte range of the token within [`Data::source()`]
    ///
    /// [`Data::source()`]: crate::lexer::Data::source()
    range: Range<usize>,
}

impl<Kind> Token<Kind> {
    /// Creates a new [`Token`] with the given `kind` and byte `range`
    pub fn new(kind: Kind, range: Range<usize>) -> Self {
        Self { kind, range }
    }
}

impl<Kind: InnerRange> Token<Kind> {
    /// Returns the inner content `range`, excluding delimiters
    ///
    /// For more information see [`crate::traits::InnerRange`].
    pub fn inner_range(&self) -> Option<Range<usize>> {
        self.kind.inner_range(self.range())
    }
}
