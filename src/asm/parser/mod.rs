use crate::asm::lexer::{Span, Token, TokenKind};
use crate::instructions::{Data, Imm, Instruction, Label, Opcode, Register, Type};
use crate::utils::Either;
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
    #[error("Expected identifier at {}..{} but found {0:?}", .1.col_start, .1.col_end)]
    ExpectedIdentifier(TokenKind, Span),
    #[error("Expected number at {}..{} but found {0:?}", .1.col_start, .1.col_end)]
    ExpectedNumber(TokenKind, Span),
    #[error("Expected delimiter at {}..{} but found {0:?}", .1.col_start, .1.col_end)]
    ExpectedDelimiter(TokenKind, Span),
    #[error("Expected percent sign at {}..{} but found {0:?}", .1.col_start, .1.col_end)]
    ExpectedPercentSign(TokenKind, Span),
    #[error("Expected colon at {}..{} but found {0:?}", .1.col_start, .1.col_end)]
    ExpectedColon(TokenKind, Span),
    #[error("Unexpected EOF")]
    UnexpectedEof,
    #[error("Expected Right Bracket but found {0:?} at {1:?}")]
    ExpectedRightBracket(TokenKind, Span),
    #[error("Expected Left Bracket but found {0:?} at {1:?}")]
    ExpectedLeftBracket(TokenKind, Span),
    #[error("Expected Signed or Unsigned but found {0:?} at {1:?}")]
    ExpectedSign(TokenKind, Span),
    #[error("Invalid Imm Type found {0:?} at {1:?}")]
    InvalidImmType(TokenKind, Span),
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
                name,
                span: label_token.span,
                def: true,
            }),
            opcode,
            span,
        }
    }

    pub fn _span(&self) -> Span {
        let Some(label) = &self.label else {
            return self.span;
        };
        Span::from((label.span, self.span))
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

    fn peek_kind(&mut self) -> Option<&TokenKind> {
        self.peek().map(|t| &t.kind)
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

    fn parse_reg(&mut self) -> Result<Register> {
        let _ = self.consume(&[TokenKind::PercentSign], ParserError::ExpectedPercentSign)?;
        let Some(token) = self.next() else {
            bail!(ParserError::UnexpectedEof);
        };
        let TokenKind::Number(reg) = token.kind else {
            bail!(ParserError::ExpectedNumber(token.kind, token.span));
        };

        Ok(Register::try_from((reg as u8, token.span))?)
    }

    fn parse_type(&mut self) -> Result<Type> {
        let _ = self.consume(&[TokenKind::LeftBracket], ParserError::ExpectedLeftBracket)?;
        let Some(sign_token) = self.next() else {
            bail!(ParserError::UnexpectedEof);
        };
        let TokenKind::Identifier(ref t) = sign_token.kind else {
            bail!(ParserError::ExpectedIdentifier(
                sign_token.kind,
                sign_token.span
            ));
        };

        let sign = &t[0..1];
        let bits = &t[1..].parse::<u8>()?;

        let _ = self.consume(
            &[TokenKind::RightBracket],
            ParserError::ExpectedRightBracket,
        )?;
        let r#type = match sign.to_lowercase().as_str() {
            "i" => Type::I(*bits),
            "u" => Type::U(*bits),
            _ => bail!(ParserError::ExpectedSign(sign_token.kind, sign_token.span)),
        };
        Ok(r#type)
    }

    fn parse_value(&mut self, t: Type) -> Result<Vec<u8>> {
        let token = match self.next() {
            Some(token) => token,
            None => bail!(ParserError::UnexpectedEof),
        };

        let value = match token.kind {
            TokenKind::Number(value) => value,
            _ => bail!(ParserError::ExpectedNumber(token.kind, token.span)),
        };

        match t {
            Type::U(8) | Type::I(8) => Ok(vec![value as u8]),
            Type::U(16) | Type::I(16) => Ok(vec![value as u8, (value >> 8) as u8]),
            Type::U(32) | Type::I(32) => Ok(vec![
                value as u8,
                (value >> 8) as u8,
                (value >> 16) as u8,
                (value >> 24) as u8,
            ]),
            Type::U(64) | Type::I(64) => Ok(vec![
                value as u8,
                (value >> 8) as u8,
                (value >> 16) as u8,
                (value >> 24) as u8,
                // TODO: This can never be 64 bits unless we can change the TokenKind::Number to
                // hold a u64 or more
                // (value >> 32) as u8,
                // (value >> 40) as u8,
                // (value >> 48) as u8,
                // (value >> 56) as u8,
            ]),
            Type::Void => bail!(ParserError::InvalidImmType(token.kind, token.span)),
            _ => unimplemented!("{:?} is not implemented", t),
        }
    }

    fn parse_reg_imm(&mut self, token: Token, opcode: Opcode) -> Result<()> {
        let r#type = self.parse_type()?;
        let reg = self.parse_reg()?;
        let value = self.parse_value(r#type)?;
        let end_span = self
            .consume(&[TokenKind::Delimiter], ParserError::ExpectedDelimiter)?
            .span;
        let span = Span::from((token.span, end_span));
        let instruction = Instruction {
            opcode,
            r#type,
            data: Data::Imm(reg, Imm(value)),
        };
        let text = self.create_text(instruction, span);
        self.push_text(text);
        Ok(())
    }

    fn _parse_reg_2(&mut self, token: Token, opcode: Opcode) -> Result<()> {
        let r#type = self.parse_type()?;
        let des = self.parse_reg()?;
        let lhs = self.parse_reg()?;
        let end_span = self
            .consume(&[TokenKind::Delimiter], ParserError::ExpectedDelimiter)?
            .span;
        let span = Span::from((token.span, end_span));
        let instruction = Instruction {
            opcode,
            r#type,
            data: Data::Reg2(des, lhs),
        };
        let text = self.create_text(instruction, span);
        self.push_text(text);
        Ok(())
    }

    fn parse_reg_3(&mut self, token: Token, opcode: Opcode) -> Result<()> {
        let r#type = self.parse_type()?;
        let des = self.parse_reg()?;
        let lhs = self.parse_reg()?;
        let rhs = self.parse_reg()?;
        let end_span = self
            .consume(&[TokenKind::Delimiter], ParserError::ExpectedDelimiter)?
            .span;

        let span = Span::from((token.span, end_span));
        let instruction = Instruction {
            opcode,
            r#type,
            data: Data::Reg3(des, lhs, rhs),
        };
        let text = self.create_text(instruction, span);
        self.push_text(text);
        Ok(())
    }

    fn parse_reg_1(&mut self, token: Token, opcode: Opcode) -> Result<()> {
        let r#type = self.parse_type()?;
        let reg = self.parse_reg()?;
        let end_span = self
            .consume(&[TokenKind::Delimiter], ParserError::ExpectedDelimiter)?
            .span;
        let span = Span::from((token.span, end_span));
        let instruction = Instruction {
            opcode,
            r#type,
            data: Data::Reg1(reg),
        };
        let text = self.create_text(instruction, span);
        self.push_text(text);
        Ok(())
    }

    fn parse_no_args(&mut self, token: Token, opcode: Opcode) -> Result<()> {
        let end_span = self
            .consume(&[TokenKind::Delimiter], ParserError::ExpectedDelimiter)?
            .span;
        let span = Span::from((token.span, end_span));
        let instruction = Instruction {
            opcode,
            r#type: Type::Void,
            data: Data::NoArgs,
        };
        let text = self.create_text(instruction, span);
        self.push_text(text);
        Ok(())
    }

    fn parse_reg_2_label(&mut self, token: Token, opcode: Opcode) -> Result<()> {
        let r#type = Type::Void;
        let lhs = self.parse_reg()?;
        let rhs = self.parse_reg()?;
        let label = self.parse_label()?;
        let end_span = self
            .consume(&[TokenKind::Delimiter], ParserError::ExpectedDelimiter)?
            .span;
        let span = Span::from((token.span, end_span));
        let instruction = Instruction {
            opcode,
            r#type,
            data: Data::RegLabel(lhs, rhs, Either::Left(label)),
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

    fn parse_label(&mut self) -> Result<Label> {
        let Some(token) = self.next() else {
            bail!(ParserError::UnexpectedEof);
        };
        let TokenKind::Identifier(name) = token.kind else {
            bail!(ParserError::ExpectedIdentifier(token.kind, token.span));
        };
        Ok(Label {
            name,
            span: token.span,
            def: false,
        })
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
                TokenKind::KeywordPush => self.parse_reg_1(token, Opcode::Push),
                TokenKind::KeywordPop => self.parse_reg_1(token, Opcode::Pop),
                TokenKind::KeywordLoad => self.parse_reg_imm(token, Opcode::Load),
                TokenKind::KeywordAdd => self.parse_reg_3(token, Opcode::Add),
                TokenKind::KeywordSub => self.parse_reg_3(token, Opcode::Sub),
                TokenKind::KeywordDiv => self.parse_reg_3(token, Opcode::Div),
                TokenKind::KeywordMul => self.parse_reg_3(token, Opcode::Mul),
                TokenKind::KeywordEq => self.parse_reg_3(token, Opcode::Eq),
                TokenKind::KeywordInc => self.parse_reg_1(token, Opcode::Inc),
                TokenKind::KeywordJne => self.parse_reg_2_label(token, Opcode::Jne),
                TokenKind::KeywordHult => self.parse_no_args(token, Opcode::Hult),
                TokenKind::KeywordPrintReg => self.parse_reg_1(token, Opcode::PrintReg),
                TokenKind::KeywordAnd => self.parse_reg_3(token, Opcode::And),
                TokenKind::KeywordOr => self.parse_reg_3(token, Opcode::Or),
                TokenKind::Identifier(_) if matches!(self.peek_kind(), Some(&TokenKind::Colon)) => {
                    self.label = Some(token);
                    let _ = self.consume(&[TokenKind::Colon], ParserError::ExpectedColon)?;
                    continue;
                }
                TokenKind::Delimiter => continue,
                _ => todo!("report error {:?}", kind),
            };
            let Err(e) = maybe_error else {
                continue;
            };
            self.errors.push(e);
        }
        if self.entry_point.is_none() {
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
    snapshot!(main, "../samples/main.asm");
}
