#![warn(missing_docs)]
//! ![logo](https://github.com/mnmun/a_bc/blob/main/logo.png?raw=true)
//!
//! A minimal set of tools for building small simple lexers.
//!
//! ## At your service
//!
//! This crate provides core building blocks - a cursor-based [`Lexer`] and
//! [`Token`] with a user-defined kind - so you only have to write the
//! token-recognition logic.
//!
//! ## How do I use it?
//!
//! Let's write a simple usage example in which the lexer distinguishes strings
//! separated by commas. Commas and strings can be separated by any number of
//! ASCII whitespace characters.
//!
//! ```rust
//! use std::ops::Range;
//! use pretty_assertions::assert_eq;
//!
//! use a_bc::{
//!     error,
//!     lexer::{Lexer, Builder},
//!     token::Token,
//!     traits::KindBounds,
//! };
//!
//! // Define enum for token kinds.
//! #[derive(PartialEq, Clone, Copy, Debug)]
//! pub enum Kind {
//!     Comma,
//!     String,
//! }
//!
//! // `Kind` must satisfy the `KindBounds` trait
//! #[allow(dead_code)]
//! fn assert_properties() {
//!     use a_bc::traits::KindBounds;
//!     fn check<T: KindBounds>() {}
//!     check::<Kind>();
//! }
//!
//! // Make newtype for our specific lexer
//! pub struct DemoLexer<'a>(Lexer<'a>);
//!
//! // To easily iterate through tokens let's implement the `Iterator` trait
//! impl<'a> Iterator for DemoLexer<'a> {
//!     type Item = Token<Kind>;
//!
//!     fn next(&mut self) -> Option<Self::Item> {
//!         // For multithread and concurrent use it is useful to be able to
//!         // shut the lexer down mid-scan, e.g. the application exit. Note
//!         // that the same cancellation token is checked again below, during
//!         // the potentially time-consuming token creation.
//!         if self.0.cancel().is_cancelled() {
//!             return None;
//!         }
//!
//!         // Skip whitespaces between tokens (spaces, newlines, tabs, etc)
//!         self.0.skip_whitespace();
//!
//!         let token = self.0.cursor().byte().and_then(|byte| {
//!             let (kind, range): (Kind, Range<usize>) = match byte {
//!                 // If the current byte is a comma, the token is exactly one
//!                 // byte long
//!                 b',' => (
//!                     Kind::Comma,
//!                     *self.0.cursor().position()..self.0.cursor().position() + 1
//!                 ),
//!                 // Otherwise, consume every byte until the end of the source
//!                 // or the next comma
//!                 _ => {
//!                     // Remember initial start position
//!                     let start = *self.0.cursor().position();
//!
//!                     while let Some(next_byte) = self.0.peek_next_byte() {
//!                         // Another cancellation token check. Although this
//!                         // loop is very fast, the check matters for the
//!                         // time-consuming token computations you could write
//!                         // here
//!                         if self.0.cancel().is_cancelled() {
//!                             break;
//!                         }
//!
//!                         if next_byte == b',' {
//!                             // Break on comma
//!                             break;
//!                         } else {
//!                             // Or read next byte
//!                             self.0.read_next_byte();
//!                         }
//!                     }
//!
//!                     (
//!                         Kind::String,
//!                         start..self.0.cursor().position() + 1
//!                     )
//!                 },
//!             };
//!
//!             Some(Token::new(kind, range))
//!         });
//!
//!         // Read next byte
//!         self.0.read_next_byte();
//!
//!         token
//!     }
//! }
//!
//! // Now it's time to test
//! let source = b"some, kind,of, s o u r c e";
//! // Chars idx:  01234567890123456789012345
//!
//! let mut lexer = DemoLexer(Builder::new(source).build().unwrap());
//! let token = lexer.next().unwrap();
//! assert_eq!(token.kind(), &Kind::String);
//! assert_eq!(token.range(), &Range { start: 0usize, end: 4usize });
//! assert_eq!(&source[token.range().clone()], b"some");
//!
//! let token = lexer.next().unwrap();
//! assert_eq!(token.kind(), &Kind::Comma);
//! assert_eq!(token.range(), &Range { start: 4usize, end: 5usize });
//! assert_eq!(&source[token.range().clone()], b",");
//!
//! let token = lexer.next().unwrap();
//! assert_eq!(token.kind(), &Kind::String);
//! assert_eq!(token.range(), &Range { start: 6usize, end: 10usize });
//! assert_eq!(&source[token.range().clone()], b"kind");
//!
//! let token = lexer.next().unwrap();
//! assert_eq!(token.kind(), &Kind::Comma);
//! assert_eq!(token.range(), &Range { start: 10usize, end: 11usize });
//! assert_eq!(&source[token.range().clone()], b",");
//!
//! let token = lexer.next().unwrap();
//! assert_eq!(token.kind(), &Kind::String);
//! assert_eq!(token.range(), &Range { start: 11usize, end: 13usize });
//! assert_eq!(&source[token.range().clone()], b"of");
//!
//! let token = lexer.next().unwrap();
//! assert_eq!(token.kind(), &Kind::Comma);
//! assert_eq!(token.range(), &Range { start: 13usize, end: 14usize });
//! assert_eq!(&source[token.range().clone()], b",");
//!
//! let token = lexer.next().unwrap();
//! assert_eq!(token.kind(), &Kind::String);
//! assert_eq!(token.range(), &Range { start: 15usize, end: 26usize });
//! assert_eq!(&source[token.range().clone()], b"s o u r c e");
//! ```
//!
//! This is a really simple example just to show the basics. [Here] you could
//! find more complex and useful example of a JSON lexer.
//!
//! ## License
//!
//! [MIT](https://github.com/mnmun/a_bc/tree/main/LICENSE)
//!
//! [`Lexer`]: crate::lexer::Lexer
//! [`Token`]: crate::token::Token
//! [Here]: https://github.com/mnmun/a_bc/tree/main/json/lib.rs

pub mod cancel;
pub mod error;
pub mod lexer;
pub mod token;
pub mod traits;
pub mod utils;
