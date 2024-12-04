use crate::lexer::token::{Span, Token};
use std::any::Any;

macro_rules! tokens {
    ($($name:ident, )* $(,)?) => {
        $(
        #[derive(Debug, PartialEq, Clone)]
        pub struct $name {
            lexeme: String,
            span: Span,
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
    Function,
    Return,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Colon,
    Plus,
    Semicolon,
    Comma,
    Number,
    Identifier,
    BBString,
    InvalidToken,
}
