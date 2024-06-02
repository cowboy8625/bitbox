use std::{iter::Peekable, str::Chars};

pub fn lex(src: &str) -> Vec<Token> {
    Lexer::new(src).lex()
}

fn is_hex_digit(c: char) -> bool {
    matches!(c, '0'..='9' | 'a'..='f' | 'A'..='F')
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct Span {
    pub row_start: usize,
    pub row_end: usize,
    pub col_start: usize,
    pub col_end: usize,
    pub byte_start: usize,
    pub byte_end: usize,
}

impl From<(Span, Span)> for Span {
    fn from((start, end): (Span, Span)) -> Self {
        Self {
            row_start: start.row_start,
            row_end: end.row_end,
            col_start: start.col_start,
            col_end: end.col_end,
            byte_start: start.byte_start,
            byte_end: end.byte_end,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TokenKind {
    KeywordLoad,
    KeywordStore,
    KeywordAloc,
    KeywordPush,
    KeywordPop,
    KeywordAdd,
    KeywordSub,
    KeywordDiv,
    KeywordMul,
    KeywordEq,
    KeywordJne,
    KeywordInc,
    KeywordHult,
    KeywordPrintReg,
    KeywordCall,
    KeywordAnd,
    KeywordOr,
    KeywordReturn,
    KeywordSyscall,
    Number(u64),
    Identifier(String),
    Colon,
    Comma,
    Period,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    PercentSign,
    Ampersand,
    Equal,
    Delimiter,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

struct Lexer<'a> {
    src: Peekable<Chars<'a>>,
    span: Span,
    tokens: Vec<Token>,
    current: Option<char>,
    last_char: Option<char>,
}

impl<'a> Lexer<'a> {
    fn new(src: &'a str) -> Self {
        Self {
            src: src.chars().peekable(),
            span: Span::default(),
            tokens: Vec::new(),
            current: None,
            last_char: None,
        }
    }

    fn next(&mut self) -> Option<char> {
        self.last_char = self.current;
        self.current = self.src.next();
        match self.current {
            Some(c) if matches!(self.last_char, Some('\n')) => {
                self.span.row_end += 1;
                self.span.col_start = 0;
                self.span.col_end = 1;
                self.span.byte_end += 1;
                Some(c)
            }
            Some(c) => {
                self.span.col_end += 1;
                self.span.byte_end += 1;
                Some(c)
            }
            None => None,
        }
    }

    fn next_if(&mut self, pred: impl Fn(char) -> bool) -> Option<char> {
        match self.peek() {
            Some(c) if pred(c) => self.next(),
            _ => None,
        }
    }

    fn peek(&mut self) -> Option<char> {
        self.src.peek().copied()
    }

    fn add_token(&mut self, kind: TokenKind) {
        let span = self.span();
        self.tokens.push(Token { kind, span });
    }

    fn add_token_with_span(&mut self, kind: TokenKind, span: Span) {
        self.tokens.push(Token { kind, span });
    }

    fn span(&mut self) -> Span {
        let span = self.span;
        self.span = Span {
            row_start: span.row_end,
            col_start: span.col_end,
            byte_start: span.byte_end,
            ..span
        };
        span
    }

    fn lex_number(&mut self, c: char) {
        let mut number = c.to_string();
        while let Some(c) = self.next_if(|c| c.is_ascii_digit()) {
            number.push(c);
        }
        let span = self.span();
        self.add_token_with_span(
            TokenKind::Number(number.parse().expect("invalid number")),
            span,
        );
    }

    fn lex_hex_number(&mut self) {
        let mut number = "".to_string();
        self.next();
        while let Some(c) = self.next_if(|c| is_hex_digit(c) || c == '_') {
            if c == '_' {
                continue;
            }
            number.push(c);
        }
        let span = self.span();
        self.add_token_with_span(
            TokenKind::Number(u64::from_str_radix(&number, 16).expect("invalid number")),
            span,
        );
    }

    fn lex_bin_number(&mut self) {
        let mut number = "".to_string();
        self.next();
        while let Some(c) = self.next_if(|c| c == '0' || c == '1' || c == '_') {
            if c == '_' {
                continue;
            }
            number.push(c);
        }
        let span = self.span();
        self.add_token_with_span(
            TokenKind::Number(u64::from_str_radix(&number, 2).expect("invalid number")),
            span,
        );
    }

    fn lex_identifier(&mut self, c: char) {
        let mut identifier = c.to_string();
        while let Some(c) = self.next_if(|c| c.is_ascii_alphanumeric() || c == '_') {
            identifier.push(c);
        }

        let kind = match identifier.as_str() {
            "load" => TokenKind::KeywordLoad,
            "store" => TokenKind::KeywordStore,
            "aloc" => TokenKind::KeywordAloc,
            "push" => TokenKind::KeywordPush,
            "pop" => TokenKind::KeywordPop,
            "add" => TokenKind::KeywordAdd,
            "sub" => TokenKind::KeywordSub,
            "div" => TokenKind::KeywordDiv,
            "mul" => TokenKind::KeywordMul,
            "inc" => TokenKind::KeywordInc,
            "eq" => TokenKind::KeywordEq,
            "jne" => TokenKind::KeywordJne,
            "hult" => TokenKind::KeywordHult,
            "printreg" => TokenKind::KeywordPrintReg,
            "call" => TokenKind::KeywordCall,
            "and" => TokenKind::KeywordAnd,
            "or" => TokenKind::KeywordOr,
            "return" => TokenKind::KeywordReturn,
            "syscall" => TokenKind::KeywordSyscall,
            _ => TokenKind::Identifier(identifier),
        };
        let span = self.span();
        self.add_token_with_span(kind, span);
    }

    fn comment(&mut self) {
        while let Some(_) = self.next_if(|c| c != '\n') {}
        self.span();
    }

    fn lex(mut self) -> Vec<Token> {
        while let Some(c) = self.next() {
            match c {
                '0' if self.peek() == Some('x') => self.lex_hex_number(),
                '0' if self.peek() == Some('b') => self.lex_bin_number(),
                '0'..='9' => self.lex_number(c),
                'a'..='z' | 'A'..='Z' | '_' => self.lex_identifier(c),
                ' ' | '\t' => {
                    self.span();
                }
                ';' => self.comment(),
                '=' => self.add_token(TokenKind::Equal),
                '\n' => self.add_token(TokenKind::Delimiter),
                '(' => self.add_token(TokenKind::LeftParen),
                ')' => self.add_token(TokenKind::RightParen),
                '{' => self.add_token(TokenKind::LeftBrace),
                '}' => self.add_token(TokenKind::RightBrace),
                '[' => self.add_token(TokenKind::LeftBracket),
                ']' => self.add_token(TokenKind::RightBracket),
                ',' => self.add_token(TokenKind::Comma),
                '.' => self.add_token(TokenKind::Period),
                ':' => self.add_token(TokenKind::Colon),
                '%' => self.add_token(TokenKind::PercentSign),
                '&' => self.add_token(TokenKind::Ampersand),
                _ => panic!("Unexpected character: {}", c),
            }
        }
        self.tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn snapshot_lexing(input: &str) -> String {
        let tokens = lex(input);
        let mut tokens = std::collections::VecDeque::from(tokens);
        let mut output = String::new();
        for (row, line) in input.lines().enumerate() {
            output += line;
            output += "\n";
            while let Some(tok) = tokens.pop_front() {
                if tok.span.row_end != row {
                    tokens.push_front(tok);
                    break;
                }
                output += &" ".repeat(tok.span.col_start);
                output += &"^".repeat(tok.span.col_end.saturating_sub(tok.span.col_start));
                output += &format!(" {tok:?}");
                output += "\n"
            }
        }
        output
    }

    macro_rules! snapshot {
        ($name:tt, $path:tt) => {
            #[test]
            fn $name() {
                let contents = include_str!($path);
                let mut settings = insta::Settings::clone_current();
                settings.set_snapshot_path("testdata/output/");
                settings.bind(|| {
                    insta::assert_snapshot!(snapshot_lexing(contents));
                });
            }
        };
    }

    snapshot!(demo, "../samples/demo.asm");
    snapshot!(main, "../samples/main.asm");
    snapshot!(fib, "../samples/fib.asm");
}
