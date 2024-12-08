use crate::lexer::token::{Span, Token};
use std::any::Any;

macro_rules! tokens {
    ($($name:ident, )* $(,)?) => {
        $(
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $name {
            pub lexeme: String,
            pub span: Span,
        }

        impl Token for $name {
            fn new(lexeme: impl Into<String>, span: Span) -> Self {
                Self {
                    lexeme: lexeme.into(),
                    span,
                }
            }

            fn get_lexeme(&self) -> String {
                self.lexeme.clone()
            }

            fn get_span(&self) -> Span {
                self.span.clone()
            }

            fn as_any(&self) -> &dyn Any {
                self
            }
        }
        )*
    };
}

tokens! {
    BBString,
    Builtin,
    Colon,
    Comma,
    Directive,
    Dot,
    Equals,
    Identifier,
    InvalidToken,
    LeftBrace,
    LeftBracket,
    LeftParen,
    Number,
    PathSeparator,
    Plus,
    RightBrace,
    RightBracket,
    RightParen,
    Semicolon,
    Star,
}
