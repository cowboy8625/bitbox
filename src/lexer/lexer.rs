use super::token;
use crate::ast::{
    Colon, Comma, Equals, Function, Identifier, If, InvalidToken, LeftBrace, LeftParen, Number,
    Plus, Return, RightBrace, RightParen, Semicolon,
};

type Token = Box<dyn token::Token>;
pub struct Lexer<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
    span: token::Span,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().peekable(),
            span: 0..0,
        }
    }

    fn next(&mut self) -> Option<char> {
        let value = self.chars.next();
        if value.is_some() {
            self.span.end += 1;
        }
        value
    }

    fn next_if(&mut self, predicate: impl Fn(char) -> bool) -> Option<char> {
        match self.chars.peek() {
            Some(value) if predicate(*value) => self.next(),
            _ => None,
        }
    }

    fn spanned(&mut self) -> token::Span {
        let span = self.span.clone();
        self.span = self.span.end..self.span.end;
        span
    }

    fn parse_number(&mut self, value: char) -> Token {
        let mut lexeme = String::from(value);

        while let Some(value) =
            self.next_if(|value| value.is_ascii_digit() || ['.', '_'].contains(&value))
        {
            lexeme.push(value);
        }

        token::create::<Number>(lexeme, self.spanned())
    }

    fn parse_identifier(&mut self, value: char) -> Token {
        let mut lexeme = String::from(value);
        while let Some(value) = self.next_if(|value| value.is_ascii_alphanumeric() || value == '_')
        {
            lexeme.push(value);
        }

        match lexeme.as_str() {
            "function" => token::create::<Function>("function", self.spanned()),
            "return" => token::create::<Return>("return", self.spanned()),
            "if" => token::create::<If>("return", self.spanned()),
            _ => token::create::<Identifier>(lexeme, self.spanned()),
        }
    }

    fn parse(&mut self) -> Option<Token> {
        match self.next() {
            Some(value @ '0'..='9') => Some(self.parse_number(value)),
            Some(value) if value.is_ascii_alphabetic() => Some(self.parse_identifier(value)),
            Some(value) if value.is_ascii_whitespace() => {
                self.spanned();
                self.parse()
            }
            Some('+') => Some(token::create::<Plus>('+', self.spanned())),
            Some('(') => Some(token::create::<LeftParen>('(', self.spanned())),
            Some(')') => Some(token::create::<RightParen>(')', self.spanned())),
            Some('{') => Some(token::create::<LeftBrace>('(', self.spanned())),
            Some('}') => Some(token::create::<RightBrace>(')', self.spanned())),
            Some(':') => Some(token::create::<Colon>(':', self.spanned())),
            Some(';') => Some(token::create::<Semicolon>(';', self.spanned())),
            Some(',') => Some(token::create::<Comma>(',', self.spanned())),
            Some('=') => Some(token::create::<Equals>('=', self.spanned())),
            Some(value) => Some(token::create::<InvalidToken>(value, self.span.clone())),
            None => None,
        }
    }
}

impl Iterator for Lexer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.parse()
    }
}