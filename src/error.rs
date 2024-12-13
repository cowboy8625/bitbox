use crate::lexer::token::{Span, Token, TokenKind};
use std::fmt::Write;

const INSTRUCTION_LIST: &[&str] = &["add", "call", "ret", "sub"];

#[derive(Debug)]
pub enum BitBoxError {
    UnexpectedToken {
        expected: TokenKind,
        actual: Token,
        help: Option<String>,
    },
    InvalidContantValue(Token),
    InvalidInstruction(Token),
    InvalidToken(Token),
    UnexpectedEndOfStream,
    ExpectedTopLevelItem(Token),
    ExpectedType(Token),
}

impl BitBoxError {
    pub fn report(&self, filename: &str, src: &str) -> String {
        match self {
            Self::InvalidToken(token) => ReportBuilder::new(filename, src, &token.span)
                .with_message("invalid token found")
                .build(),
            Self::InvalidContantValue(token) => ReportBuilder::new(filename, src, &token.span)
                .with_message("invalid constant value")
                .with_note("expected a directive, number or string")
                .build(),
            Self::InvalidInstruction(token) => ReportBuilder::new(filename, src, &token.span)
                .with_message("invalid instruction")
                .with_note("expected one of: add, call, ret, sub ...")
                .build(),
            Self::UnexpectedToken {
                expected,
                actual,
                help,
            } => ReportBuilder::new(filename, src, &actual.span)
                .with_message("unexpected token")
                .with_note(format!(
                    "expected: {:?}, found: {}",
                    expected, actual.lexeme
                ))
                .build(),
            Self::UnexpectedEndOfStream => {
                ReportBuilder::new(filename, src, &(src.len().saturating_sub(1)..src.len()))
                    .with_message("unexpected end of stream")
                    .build()
            }
            Self::ExpectedTopLevelItem(token) => ReportBuilder::new(filename, src, &token.span)
                .with_message("expected a top level item")
                .with_note("function, import or constant")
                .build(),
            Self::ExpectedType(token) => ReportBuilder::new(filename, src, &token.span)
                .with_message(format!("expected a type but found {:?}", token.kind))
                .with_note("expected a type: s32, u32, f32, ...")
                .build(),
        }
    }
}

impl From<Token> for BitBoxError {
    fn from(token: Token) -> Self {
        Self::InvalidToken(token)
    }
}

pub struct ReportBuilder<'a> {
    message: String,
    filename: &'a str,
    row: usize,
    col: usize,
    problem_line: &'a str,
    note: Option<String>,
    underline: String,
}

impl<'a> ReportBuilder<'a> {
    pub fn new(filename: &'a str, src: &'a str, span: &'a Span) -> Self {
        let (row, col) = Self::get_row_col_from_span(src, span);
        let problem_line = Self::get_problem_src_line(src, row);
        let underline = Self::get_underline(col, span);

        Self {
            message: String::new(),
            filename,
            row,
            col,
            problem_line,
            note: None,
            underline,
        }
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    pub fn build(&self) -> String {
        let mut report = String::new();
        writeln!(&mut report, "{}:{}:{}", self.row, self.col, self.filename)
            .expect("failed to write report");
        if !self.message.is_empty() {
            writeln!(&mut report, " --> {}", self.message).expect("failed to write message");
        }
        writeln!(&mut report, "{:<3}| {}", self.row, self.problem_line)
            .expect("failed to write problem line");
        writeln!(&mut report, "   | {}", self.underline).expect("failed to write underline");
        if let Some(note) = &self.note {
            writeln!(&mut report, "   | = note: {}", note).expect("failed to write help");
        }
        report
    }

    fn get_row_col_from_span(src: &str, span: &Span) -> (usize, usize) {
        let row = src[..span.start].chars().filter(|&c| c == '\n').count();
        let col = span.start - src[..span.start].rfind('\n').map(|n| n + 1).unwrap_or(0);
        (row, col)
    }

    fn get_problem_src_line(src: &'a str, row: usize) -> &'a str {
        src.lines()
            .nth(row)
            .unwrap_or("Failed to extract source line")
    }

    fn get_underline(col: usize, span: &Span) -> String {
        let spacing = " ".repeat(col);
        let underline = "^".repeat(span.len());
        format!("{}{}", spacing, underline)
    }
}
