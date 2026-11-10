use pretty_assertions::assert_eq;

use std::fmt;

use a_bc::{
    error::{self, Error, Position, Relation, row_col_pos},
    lexer::{Builder, Lexer},
    token::Token,
};

fn main() {
    // Token kinds
    #[derive(PartialEq, Clone, Copy, Debug)]
    enum Kind {
        // A single character ','
        Comma,

        // A double-quoted strings, e.g. "this is a string"
        String,

        // Any character that does not match the defined kinds belongs here
        Unexpected,
    }

    impl fmt::Display for Kind {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Kind::Comma => write!(f, "Comma"),
                Kind::String => write!(f, "String"),
                Kind::Unexpected => write!(f, "Unexpected token"),
            }
        }
    }

    // `Kind` must satisfy the `KindBounds` trait
    #[allow(dead_code)]
    fn assert_bounds() {
        use a_bc::traits::KindBounds;
        fn check<T: KindBounds>() {}
        check::<Kind>();
    }

    // A newtype wrapper for the specific lexer
    #[repr(transparent)]
    struct DemoLexer<'source>(Lexer<'source>);

    // Implements the `Iterator` trait to enable straightforward iteration
    // over tokens
    impl Iterator for DemoLexer<'_> {
        type Item = Result<Token<Kind>, Error<Kind>>;

        fn next(&mut self) -> Option<Self::Item> {
            // In multithread and concurrent scenarios, it may be necessary to
            // shut down the lexer in the middle of a scan, e.g. upon
            // application exit.
            if self.0.flag().is_cancelled() {
                return None;
            }

            // Skips whitespace characters between tokens (spaces, newlines,
            // tabs, etc.)
            self.0.skip_whitespace();

            let token: Option<Result<Token<Kind>, Error<Kind>>> =
                self.0.cursor().byte().and_then(|byte: u8| {
                    Some(match byte {
                        // If the current byte is a comma, the token is exactly one
                        // byte long
                        b',' => Ok(Token::new(
                            Kind::Comma,
                            *self.0.cursor().position()
                                ..self.0.cursor().position() + 1,
                        )),
                        // If the current byte is a quote, consumes every byte until
                        // the closing quote or the end of the source
                        b'"' => {
                            // Remember the initial start position
                            let start: usize = *self.0.cursor().position();

                            loop {
                                // Another cancellation flag check
                                if self.0.flag().is_cancelled() {
                                    break;
                                }

                                if let Some(current_byte) =
                                    self.0.read_next_byte()
                                {
                                    // Break on closing quote
                                    if current_byte == b'"' {
                                        break;
                                    }
                                } else {
                                    // In this case closing quote was not found
                                    return Some(Err(
                                        error::Token::PairNotFound {
                                            opening: "\"".into(),
                                            closing: "\"".into(),
                                            location: (
                                                Relation::After,
                                                row_col_pos(
                                                    &self.0.data().source()
                                                        [0..=start],
                                                ),
                                            ),
                                        }
                                        .into(),
                                    ));
                                }
                            }

                            Ok(Token::new(
                                Kind::String,
                                start..self.0.cursor().position() + 1,
                            ))
                        }
                        // In this case an unexpected character was encountered
                        _ => {
                            let position: usize = *self.0.cursor().position();

                            Err(error::Token::ExpectedButGot {
                                expected: [Kind::Comma, Kind::String].into(),
                                got: Some(Kind::Unexpected),
                                location: (
                                    Relation::At,
                                    row_col_pos(
                                        &self.0.data().source()[0..=position],
                                    ),
                                ),
                            }
                            .into())
                        }
                    })
                });

            // Read next byte
            self.0.read_next_byte();

            token
        }
    }

    // Valid source data:
    let valid_source: &'static [u8] = br#" "valid" , "source" "#;

    let mut lexer: DemoLexer<'static> =
        DemoLexer(Builder::new(valid_source).build().unwrap());

    let token: Token<Kind> = lexer.next().unwrap().unwrap();
    assert_eq!(token.kind(), &Kind::String);
    assert_eq!(&valid_source[token.range().clone()], br#""valid""#);

    let token: Token<Kind> = lexer.next().unwrap().unwrap();
    assert_eq!(token.kind(), &Kind::Comma);
    assert_eq!(&valid_source[token.range().clone()], b",");

    let token: Token<Kind> = lexer.next().unwrap().unwrap();
    assert_eq!(token.kind(), &Kind::String);
    assert_eq!(&valid_source[token.range().clone()], br#""source""#);

    // Lexer cannot be created over empty source data:
    let empty_source: &'static [u8] = b"";
    assert!(
        Builder::new(empty_source)
            .build()
            .is_err_and(|e| e == error::Lexer::SourceIsEmpty)
    );

    // Missing closing double quotation mark results in an error:
    let source_with_missing_quote: &'static [u8] =
        br#" "missing closing quote "#;

    let mut lexer: DemoLexer<'static> =
        DemoLexer(Builder::new(source_with_missing_quote).build().unwrap());

    assert!(lexer.next().unwrap().is_err_and(|e| {
        e == error::Token::PairNotFound {
            opening: "\"".into(),
            closing: "\"".into(),
            location: (Relation::After, Position::new(1, 2)),
        }
        .into()
    }));

    // Source data containing unexpected tokens results in an error:
    let source_with_unexpected_token: &'static [u8] = b" unexpected token ";

    let mut lexer: DemoLexer<'static> =
        DemoLexer(Builder::new(source_with_unexpected_token).build().unwrap());

    assert!(lexer.next().unwrap().is_err_and(|e| {
        e == error::Token::ExpectedButGot {
            expected: [Kind::Comma, Kind::String].into(),
            got: Some(Kind::Unexpected),
            location: (Relation::At, Position::new(1, 2)),
        }
        .into()
    }));
}
