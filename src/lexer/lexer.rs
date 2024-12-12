use super::token::{Directive, Instruction, Keyword, Span, Token, TokenKind};

pub struct Lexer<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
    span: Span,
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

    fn spanned(&mut self, kind: TokenKind, lexeme: impl Into<String>) -> Token {
        let span = self.span.clone();
        self.span = self.span.end..self.span.end;
        Token {
            span,
            kind,
            lexeme: lexeme.into(),
        }
    }

    fn parse_number(&mut self, value: char) -> Token {
        let mut lexeme = String::from(value);

        while let Some(value) =
            self.next_if(|value| value.is_ascii_digit() || ['.', '_'].contains(&value))
        {
            lexeme.push(value);
        }

        self.spanned(TokenKind::Number, lexeme)
    }

    fn parse_identifier(&mut self, value: char) -> Token {
        let mut lexeme = String::from(value);
        while let Some(value) = self.next_if(|value| value.is_ascii_alphanumeric() || value == '_')
        {
            lexeme.push(value);
        }

        match lexeme.as_str() {
            "import" => self.spanned(TokenKind::Keyword(Keyword::Import), lexeme),
            "const" => self.spanned(TokenKind::Keyword(Keyword::Const), lexeme),
            "function" => self.spanned(TokenKind::Keyword(Keyword::Function), lexeme),
            "public" => self.spanned(TokenKind::Keyword(Keyword::Public), lexeme),
            _ => self.spanned(TokenKind::Identifier, lexeme),
        }
    }

    fn parse_builtin(&mut self) -> Token {
        let mut lexeme = String::from('@');
        while let Some(value) = self.next_if(|value| value.is_ascii_alphanumeric() || value == '_')
        {
            lexeme.push(value);
        }

        let kind = match lexeme.as_str() {
            "@add" => Instruction::Add,
            "@sub" => Instruction::Sub,
            "@call" => Instruction::Call,
            "@ret" => Instruction::Ret,
            _ => return self.spanned(TokenKind::InvalidToken, lexeme),
        };

        self.spanned(TokenKind::Instruction(kind), lexeme)
    }

    fn parse_string(&mut self) -> Token {
        let mut lexeme = String::from('#');
        while let Some(value) = self.next() {
            lexeme.push(value);
            if lexeme.ends_with("\"#") {
                break;
            }
        }
        let lexeme = lexeme[2..lexeme.len() - 2].replace("\\n", "\n");
        self.spanned(TokenKind::String, lexeme)
    }

    fn parse_directive(&mut self) -> Token {
        let mut lexeme = String::from('.');
        while let Some(value) = self.next_if(|value| value.is_ascii_alphanumeric() || value == '_')
        {
            lexeme.push(value);
        }

        let kind = match lexeme.as_str() {
            ".len" => Directive::Len,
            _ => return self.spanned(TokenKind::InvalidToken, lexeme),
        };

        self.spanned(TokenKind::Directive(kind), lexeme)
    }

    fn skip(&mut self) -> Option<Token> {
        self.spanned(TokenKind::InvalidToken, ' ');
        self.parse()
    }

    fn parse(&mut self) -> Option<Token> {
        match self.next() {
            Some(value @ '0'..='9') => Some(self.parse_number(value)),
            Some(value) if value.is_ascii_alphabetic() => Some(self.parse_identifier(value)),
            Some(value) if value.is_ascii_whitespace() => self.skip(),
            Some('#') if self.chars.peek() == Some(&'"') => Some(self.parse_string()),
            Some('.') if self.chars.peek() != Some(&' ') => Some(self.parse_directive()),
            Some(':') if self.chars.peek() == Some(&':') => {
                self.next();
                Some(self.spanned(TokenKind::PathSeparator, "::"))
            }
            Some('@') => Some(self.parse_builtin()),
            Some('+') => Some(self.spanned(TokenKind::Plus, '+')),
            Some('(') => Some(self.spanned(TokenKind::LeftParen, '(')),
            Some(')') => Some(self.spanned(TokenKind::RightParen, ')')),
            Some('{') => Some(self.spanned(TokenKind::LeftBrace, '{')),
            Some('}') => Some(self.spanned(TokenKind::RightBrace, '}')),
            Some('[') => Some(self.spanned(TokenKind::LeftBracket, '[')),
            Some(']') => Some(self.spanned(TokenKind::RightBracket, ']')),
            Some(':') => Some(self.spanned(TokenKind::Colon, ':')),
            Some(';') => Some(self.spanned(TokenKind::Semicolon, ';')),
            Some(',') => Some(self.spanned(TokenKind::Comma, ',')),
            Some('=') => Some(self.spanned(TokenKind::Equals, '=')),
            Some('.') => Some(self.spanned(TokenKind::Dot, '.')),
            Some('*') => Some(self.spanned(TokenKind::Star, '*')),
            Some(value) => Some(self.spanned(TokenKind::InvalidToken, value)),
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
