//! # Lexed [`token`]
//!
//! ![](https://github.com/mnmun/images/blob/main/trail.png?raw=true)
//!
//! Provides a [`token`] - a lexed unit of text.
//!
//! ---
//!
//! See the [`crate`] documentation for more information.
//!
//! [`token`]: crate::Token

use getset::Getters;
use std::{fmt, ops::Range};

use crate::traits::{InnerRange, KindBounds};

/// # Lexed `token`
///
/// ![](https://github.com/mnmun/images/blob/main/trail.png?raw=true)
///
/// User-defined `kind` and the byte `range` that it occupies within the
/// [`source`] data.
///
/// The `Kind` type parameter must satisfy the [`KindBounds`] trait.
///
/// ## Creation
///
/// Can be created using the [`Token::new()`] method:
///
/// ```rust
/// # use pretty_assertions::assert_eq;
/// #
/// use std::fmt;
///
/// use a_bc::Token;
///
/// #[derive(PartialEq, Clone, Copy, Debug)]
/// enum SomeKind {
///     Foo,
///     Bar,
/// }
///
/// impl fmt::Display for SomeKind {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
///         match self {
///             SomeKind::Foo => write!(f, "Foo"),
///             SomeKind::Bar => write!(f, "Bar"),
///         }
///     }
/// }
///
/// let token: Token<SomeKind> = Token::new(SomeKind::Foo, 1..5);
/// assert_eq!(*token.kind(), SomeKind::Foo);
/// assert_eq!(*token.range(), 1..5);
/// ```
///
/// ---
///
/// See the [`module`] documentation for more information.
///
/// [`source`]: crate::lexer::Data::source()
/// [`new()`]: crate::Token::new()
/// [`KindBounds`]: crate::traits::KindBounds
/// [`module`]: crate::token
#[derive(Debug, PartialEq, Clone, Getters)]
#[getset(get = "pub")]
pub struct Token<Kind: KindBounds> {
    /// # The user-defined token `kind`
    ///
    /// See the [`token`] documentation for more information.
    ///
    /// [`token`]: crate::Token
    kind: Kind,

    /// # The byte `range` occupied by the [`token`] within the [`source`] data
    ///
    /// See the [`token`] documentation for more information.
    ///
    /// [`token`]: crate::Token
    /// [`source`]: crate::lexer::Data::source()
    range: Range<usize>,
}

impl<Kind: KindBounds> Token<Kind> {
    /// # Creates a new [`token`] with the given `kind` and byte `range`
    ///
    /// See the [`token`] documentation for more information.
    ///
    /// [`token`]: crate::Token
    pub fn new(kind: Kind, range: Range<usize>) -> Self {
        Self { kind, range }
    }
}

impl<Kind: KindBounds + InnerRange> Token<Kind> {
    /// # Returns the inner content `range`, excluding delimiters
    ///
    /// See the [`InnerRange`] documentation for more information.
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
