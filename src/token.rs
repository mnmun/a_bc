//! # Lexed [`token`]
//!
//! ![](https://github.com/mnmun/images/blob/main/trail.png?raw=true)
//!
//! Provides [`token`] - a lexed unit of text.
//!
//! ---
//!
//! See the [`crate documentation`] for more information.
//!
//! [`token`]: Token
//! [`crate documentation`]: crate

use getset::Getters;
use std::{fmt, ops::Range};

use crate::traits::{InnerRange, KindBounds};

/// # Lexed `token`
///
/// ![](https://github.com/mnmun/images/blob/main/trail.png?raw=true)
///
/// Represented by a user-defined `kind` and the byte `range` that it occupies
/// within the [`source`] data.
///
/// The `Kind` type parameter must satisfy the [`KindBounds`] trait.
///
/// An instance can be obtained using the [`Token::new()`] method.
///
/// ---
///
/// See the [`module documentation`] for more information.
///
/// [`source`]: crate::lexer::Data::source()
/// [`new()`]: Token::new()
/// [`KindBounds`]: crate::traits::KindBounds
/// [`module documentation`]: crate::token
#[derive(Debug, PartialEq, Clone, Getters)]
#[getset(get = "pub")]
pub struct Token<Kind: KindBounds> {
    /// # The user-defined token kind
    kind: Kind,

    /// # The byte range occupied by the token within the [`source`] data
    ///
    /// [`source`]: crate::lexer::Data::source()
    range: Range<usize>,
}

impl<Kind: KindBounds> Token<Kind> {
    /// # Creates a new [`token`] with the given `kind` and byte `range`
    ///
    /// [`token`]: Token
    pub fn new(kind: Kind, range: Range<usize>) -> Self {
        Self { kind, range }
    }
}

impl<Kind: KindBounds + InnerRange> Token<Kind> {
    /// # Returns the inner content `range`, excluding delimiters
    ///
    /// ---
    ///
    /// See the [`InnerRange`] for more information.
    pub fn inner_range(&self) -> Option<Range<usize>> {
        self.kind.inner_range(self.range())
    }
}

impl<Kind: KindBounds> fmt::Display for Token<Kind> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} token with range ", self.kind)?;
        write!(f, "{:?}", self.range)
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use std::fmt::{self, Debug};

    use crate::Token;

    #[test]
    fn diplay() {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        enum Kind {
            Foo,
        }

        impl fmt::Display for Kind {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "Foo")
            }
        }

        let token = Token::new(Kind::Foo, 0..10);
        assert_eq!(format!("{token}"), "Foo token with range 0..10");
    }
}
