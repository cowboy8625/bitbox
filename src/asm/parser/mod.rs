use crate::asm::lexer::{Span, Token, TokenKind};
use crate::instructions::Instruction;
use anyhow::{bail, Result};
use std::iter::Peekable;
use thiserror::Error;

pub fn parse(src: &str) -> Result<Vec<Item>> {
    let tokens = super::lexer::lex(src);
    let parser = Parser::new(tokens);
    parser.parse()
}

#[derive(Debug, Error)]
struct BatchParserErrors(Vec<anyhow::Error>);

impl std::fmt::Display for BatchParserErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for e in &self.0 {
            writeln!(f, "{}", e)?;
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum ParserError {
    #[error("Unknown directive {0:?} at {}..{}", .1.col_start, .1.col_end)]
    UnknownDirective(String, Span),
    #[error("Missing entry point")]
    MissingEntryPoint,
    #[error("Invalid instruction {0:?} at {}..{}", .1.col_start, .1.col_end)]
    InvalidInstruction(TokenKind, Span),
    #[error("Expected identifier at {}..{} but found {0:?}", .1.col_start, .1.col_end)]
    ExpectedIdentifier(TokenKind, Span),
    #[error("Expected number at {}..{} but found {0:?}", .1.col_start, .1.col_end)]
    ExpectedNumber(TokenKind, Span),
    #[error("Register out of bounds at {}..{}", .1.col_start, .1.col_end)]
    RegisterOutOfBounds(u8, Span),
    #[error("Expected delimiter at {}..{} but found {0:?}", .1.col_start, .1.col_end)]
    ExpectedDelimiter(TokenKind, Span),
    #[error("Expected percent sign at {}..{} but found {0:?}", .1.col_start, .1.col_end)]
    ExpectedPercentSign(TokenKind, Span),
    #[error("Unexpected EOF")]
    UnexpectedEof,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reg {
    R0 = 0,
    R1 = 1,
    R2 = 2,
    R3 = 3,
    R4 = 4,
    R5 = 5,
    R6 = 6,
    R7 = 7,
    R8 = 8,
    R9 = 9,
    R10 = 10,
    R11 = 11,
    R12 = 12,
    R13 = 13,
    R14 = 14,
    R15 = 15,
    R16 = 16,
    R17 = 17,
    R18 = 18,
    R19 = 19,
    R20 = 20,
    R21 = 21,
    R22 = 22,
    R23 = 23,
    R24 = 24,
    R25 = 25,
    R26 = 26,
    R27 = 27,
    R28 = 28,
    R29 = 29,
    R30 = 30,
    R31 = 31,
}

impl TryFrom<(u8, Span)> for Reg {
    type Error = ParserError;

    fn try_from((value, span): (u8, Span)) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Reg::R0),
            1 => Ok(Reg::R1),
            2 => Ok(Reg::R2),
            3 => Ok(Reg::R3),
            4 => Ok(Reg::R4),
            5 => Ok(Reg::R5),
            6 => Ok(Reg::R6),
            7 => Ok(Reg::R7),
            8 => Ok(Reg::R8),
            9 => Ok(Reg::R9),
            10 => Ok(Reg::R10),
            11 => Ok(Reg::R11),
            12 => Ok(Reg::R12),
            13 => Ok(Reg::R13),
            14 => Ok(Reg::R14),
            15 => Ok(Reg::R15),
            16 => Ok(Reg::R16),
            17 => Ok(Reg::R17),
            18 => Ok(Reg::R18),
            19 => Ok(Reg::R19),
            20 => Ok(Reg::R20),
            21 => Ok(Reg::R21),
            22 => Ok(Reg::R22),
            23 => Ok(Reg::R23),
            24 => Ok(Reg::R24),
            25 => Ok(Reg::R25),
            26 => Ok(Reg::R26),
            27 => Ok(Reg::R27),
            28 => Ok(Reg::R28),
            29 => Ok(Reg::R29),
            30 => Ok(Reg::R30),
            31 => Ok(Reg::R31),
            _ => Err(ParserError::RegisterOutOfBounds(value, span)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label {
    pub name: String,
    pub span: Span,
    pub def: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    EntryPoint(Token),
    // Data(Vec<Data>),
    Text(Vec<Text>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Text {
    pub label: Option<Label>,
    pub opcode: Instruction,
    pub span: Span,
}

impl Text {
    pub fn new_opcode(opcode: Instruction, span: Span) -> Self {
        Self {
            label: None,
            opcode,
            span,
        }
    }
    pub fn new_opcode_with_label(label_token: Token, opcode: Instruction, span: Span) -> Self {
        let TokenKind::Identifier(name) = label_token.kind else {
            panic!("Expected identifier")
        };
        Self {
            label: Some(Label {
                name: name.into(),
                span: label_token.span,
                def: true,
            }),
            opcode,
            span,
        }
    }

    pub fn span(&self) -> Span {
        let Some(label) = &self.label else {
            return self.span;
        };
        let Span {
            row_start,
            col_start,
            byte_start,
            ..
        } = label.span;
        let Span {
            row_end,
            col_end,
            byte_end,
            ..
        } = self.span;
        Span {
            row_start,
            col_start,
            row_end,
            col_end,
            byte_start,
            byte_end,
        }
    }
}

struct Parser {
    tokens: Peekable<std::vec::IntoIter<Token>>,
    instructions: Vec<Item>,
    text_section: Vec<Text>,
    // data_section: Vec<Data>,
    entry_point: Option<Token>,
    label: Option<Token>,
    errors: Vec<anyhow::Error>,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens: tokens.into_iter().peekable(),
            instructions: Vec::new(),
            text_section: Vec::new(),
            // data_section: Vec::new(),
            entry_point: None,
            label: None,
            errors: Vec::new(),
        }
    }

    fn next(&mut self) -> Option<Token> {
        self.tokens.next()
    }

    fn peek(&mut self) -> Option<&Token> {
        self.tokens.peek()
    }

    fn push_text(&mut self, text: Text) {
        self.text_section.push(text)
    }

    fn create_text(&mut self, opcode: Instruction, span: Span) -> Text {
        let Some(label) = self.label.take() else {
            return Text::new_opcode(opcode, span);
        };
        Text::new_opcode_with_label(label, opcode, span)
    }

    fn consume(
        &mut self,
        kinds: &[TokenKind],
        error: impl Fn(TokenKind, Span) -> ParserError,
    ) -> Result<Token> {
        let Some(token) = self.next() else {
            bail!(ParserError::UnexpectedEof);
        };
        if !kinds.contains(&token.kind) {
            bail!(error(token.kind, token.span));
        }
        Ok(token)
    }

    fn parse_reg(&mut self) -> Result<Reg> {
        let _ = self.consume(&[TokenKind::PercentSign], ParserError::ExpectedPercentSign)?;
        let Some(token) = self.next() else {
            bail!(ParserError::UnexpectedEof);
        };
        let TokenKind::Number(reg) = token.kind else {
            bail!(ParserError::ExpectedNumber(token.kind, token.span));
        };

        Ok(Reg::try_from((reg as u8, token.span))?)
    }

    fn parse_u16(&mut self) -> Result<u16> {
        let Some(token) = self.next() else {
            bail!(ParserError::UnexpectedEof);
        };
        let TokenKind::Number(value) = token.kind else {
            bail!(ParserError::ExpectedNumber(token.kind, token.span));
        };
        Ok(value as u16)
    }

    fn parse_reg_u16(
        &mut self,
        token: Token,
        instruction: impl Fn(u8, u8, u8) -> Instruction,
    ) -> Result<()> {
        let Span {
            row_start,
            col_start,
            byte_start,
            ..
        } = token.span;
        let reg = self.parse_reg()?;
        let value = self.parse_u16()?;
        let value_upper = (value >> 8) as u8;
        let value_lower = (value & 0xff) as u8;
        let Span {
            row_end,
            col_end,
            byte_end,
            ..
        } = self
            .consume(&[TokenKind::Delimiter], ParserError::ExpectedDelimiter)?
            .span;
        let span = Span {
            row_start,
            row_end,
            col_start,
            col_end,
            byte_start,
            byte_end,
        };
        let text = self.create_text(instruction(reg as u8, value_upper, value_lower), span);
        self.push_text(text);
        Ok(())
    }

    fn parse_reg_3(
        &mut self,
        token: Token,
        instruction: impl Fn(u8, u8, u8) -> Instruction,
    ) -> Result<()> {
        let Span {
            row_start,
            col_start,
            byte_start,
            ..
        } = token.span;
        let des = self.parse_reg()?;
        let lhs = self.parse_reg()?;
        let rhs = self.parse_reg()?;
        let Span {
            row_end,
            col_end,
            byte_end,
            ..
        } = self
            .consume(&[TokenKind::Delimiter], ParserError::ExpectedDelimiter)?
            .span;

        let span = Span {
            row_start,
            row_end,
            col_start,
            col_end,
            byte_start,
            byte_end,
        };
        let text = self.create_text(instruction(des as u8, lhs as u8, rhs as u8), span);
        self.push_text(text);
        Ok(())
    }

    fn parse_no_args(&mut self, token: Token, instruction: Instruction) -> Result<()> {
        let Span {
            row_start,
            col_start,
            byte_start,
            ..
        } = token.span;
        let Span {
            row_end,
            col_end,
            byte_end,
            ..
        } = self
            .consume(&[TokenKind::Delimiter], ParserError::ExpectedDelimiter)?
            .span;
        let span = Span {
            row_start,
            row_end,
            col_start,
            col_end,
            byte_start,
            byte_end,
        };
        let text = self.create_text(instruction, span);
        self.push_text(text);
        Ok(())
    }

    fn entry_point(&mut self) -> Result<()> {
        // TODO: Make span match full entry point
        let token = self.next();

        if token
            .clone()
            .map(|t| !matches!(t.kind, TokenKind::Identifier(_)))
            .unwrap_or(true)
        {
            bail!(ParserError::ExpectedIdentifier(
                token.clone().unwrap().kind,
                token.unwrap().span
            ));
        }
        self.entry_point = token;
        let _ = self.consume(&[TokenKind::Delimiter], ParserError::ExpectedDelimiter)?;
        Ok(())
    }

    fn parse_directive(&mut self, token: Token) -> Result<()> {
        let identifier = self.next();
        let Some(Token { kind, span }) = identifier else {
            bail!(ParserError::ExpectedIdentifier(token.kind, token.span));
        };
        let TokenKind::Identifier(name) = kind else {
            bail!(ParserError::ExpectedIdentifier(kind, span));
        };
        match name.as_str() {
            "entry" => self.entry_point()?,
            _ => bail!(ParserError::UnknownDirective(name, span)),
        }
        Ok(())
    }

    fn parse(mut self) -> Result<Vec<Item>> {
        while let Some(token) = self.next() {
            let kind = token.kind.clone();
            let maybe_error = match kind {
                TokenKind::Period => self.parse_directive(token),
                TokenKind::KeywordLoadInt => self.parse_reg_u16(token, Instruction::LoadInt),
                TokenKind::KeywordAdd => self.parse_reg_3(token, Instruction::Add),
                TokenKind::KeywordHult => self.parse_no_args(token, Instruction::Hult),
                TokenKind::Delimiter => continue,
                _ => todo!("report error {:?}", kind),
            };
            let Err(e) = maybe_error else {
                continue;
            };
            self.errors.push(e);
        }
        if let None = self.entry_point {
            self.errors.push(ParserError::MissingEntryPoint.into());
        }
        if !self.errors.is_empty() {
            bail!(BatchParserErrors(self.errors));
        }
        self.instructions
            .push(Item::EntryPoint(self.entry_point.unwrap()));
        self.instructions.push(Item::Text(self.text_section));
        Ok(self.instructions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot_entry_point(line: &str, entry_item: Option<Item>) -> Result<String> {
        let mut output = String::new();
        let Some(Item::EntryPoint(Token { span, kind })) = entry_item else {
            panic!("Expected entry point but found: {:?}", entry_item);
        };
        output += line;
        output += "\n";
        output += &" ".repeat(span.col_start);
        output += &"^".repeat(span.col_end - span.col_start);
        output += &format!(" {kind:?}");
        output += "\n";
        Ok(output)
    }

    pub fn snapshot_parsing(input: &str) -> Result<String> {
        let items = parse(input)?;
        let mut line_iter = input.lines();
        let Some(line) = line_iter.next() else {
            panic!("Expected at least one line");
        };

        let mut items = std::collections::VecDeque::from(items);
        let mut output = snapshot_entry_point(line, items.pop_front())?;
        let text_section = items.pop_front();
        let Some(Item::Text(text_section)) = text_section else {
            panic!("Expected text section but found: {:?}", text_section);
        };

        let mut text_section = std::collections::VecDeque::from(text_section);
        for (row, line) in line_iter.enumerate() {
            output += line;
            output += "\n";
            while let Some(text) = text_section.pop_front() {
                if text.span.row_end != row {
                    text_section.push_front(text);
                    break;
                }
                output += &" ".repeat(text.span.col_start);
                output += &"^".repeat(text.span.col_end - text.span.col_start);
                output += &format!(" {text:?}");
                output += "\n"
            }
        }
        Ok(output)
    }

    macro_rules! snapshot {
        ($name:tt, $path:tt) => {
            #[test]
            fn $name() {
                let contents = include_str!($path);
                let mut settings = insta::Settings::clone_current();
                settings.set_snapshot_path("testdata/output/");
                settings.bind(|| {
                    insta::assert_snapshot!(snapshot_parsing(contents).unwrap());
                });
            }
        };
    }

    snapshot!(demo, "../samples/demo.asm");
}
