#[cfg(test)]
mod test;
use crate::error::BitBoxError;
use crate::lexer::token::{self, Span, Token, TokenKind};
use crate::ssa::{self, IntoSsaType};

enum TopLevel {
    Function(ssa::Function),
    Import(ssa::Import),
    Constant(ssa::Constant),
}

pub struct Parser {
    stream: std::iter::Peekable<std::vec::IntoIter<Token>>,
}

// Helpers
impl Parser {
    fn parse_visibility(&mut self) -> ssa::Visibility {
        self.stream
            .next_if(|token| token.is_keyword(token::Keyword::Public))
            .map(|_| ssa::Visibility::Public)
            .unwrap_or_default()
    }

    fn consume(&mut self, expected: TokenKind) -> Result<Token, BitBoxError> {
        match self.stream.next() {
            Some(actual) if actual.kind == expected => Ok(actual),
            Some(actual) => Err(BitBoxError::UnexpectedToken { expected, actual }),
            None => Err(BitBoxError::UnexpectedEndOfStream),
        }
    }

    fn is_peek_a(&mut self, kind: TokenKind) -> bool {
        matches!(self.stream.peek(), Some(token) if token.kind == kind)
    }

    fn end_of_stream(&mut self) -> bool {
        self.stream.peek().is_none()
    }

    fn next(&mut self) -> Result<Token, BitBoxError> {
        let Some(token) = self.stream.next() else {
            return Err(BitBoxError::UnexpectedEndOfStream);
        };
        Ok(token)
    }
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            stream: tokens.into_iter().peekable(),
        }
    }

    pub fn parse(&mut self) -> Result<ssa::Program, BitBoxError> {
        let mut imports = vec![];
        let mut functions = vec![];
        let mut constants = vec![];

        while !self.end_of_stream() {
            match self.parse_top_level()? {
                TopLevel::Import(import) => imports.push(import),
                TopLevel::Function(func) => functions.push(func),
                TopLevel::Constant(constant) => constants.push(constant),
            }
        }

        Ok(ssa::Program {
            functions,
            imports,
            constants,
        })
    }

    fn parse_top_level(&mut self) -> Result<TopLevel, BitBoxError> {
        let visibility = self.parse_visibility();
        if self.is_peek_a(TokenKind::Keyword(token::Keyword::Function)) {
            Ok(TopLevel::Function(self.parse_function(visibility)?))
        } else if self.is_peek_a(TokenKind::Keyword(token::Keyword::Import)) {
            Ok(TopLevel::Import(self.parse_import()?))
        } else if self.is_peek_a(TokenKind::Keyword(token::Keyword::Const)) {
            Ok(TopLevel::Constant(self.parse_constant()?))
        } else {
            let tok = self.next()?;
            Err(BitBoxError::ExpectedTopLevelItem(tok))
        }
    }

    fn parse_function(
        &mut self,
        visibility: ssa::Visibility,
    ) -> Result<ssa::Function, BitBoxError> {
        self.consume(TokenKind::Keyword(token::Keyword::Function))?;
        let func_name = self.consume(TokenKind::Identifier)?;
        let params = self.parse_function_params()?;
        let return_type = self.parse_type()?;
        let blocks = self.parse_function_block()?;

        Ok(ssa::Function {
            visibility,
            name: func_name.lexeme,
            params,
            return_type,
            blocks,
        })
    }

    fn parse_function_params(&mut self) -> Result<Vec<ssa::Variable>, BitBoxError> {
        self.consume(TokenKind::LeftParen)?;
        let mut params = vec![];

        let mut version = 0;
        while !self.end_of_stream() && !self.is_peek_a(TokenKind::RightParen) {
            let name = self.consume(TokenKind::Identifier)?;
            self.consume(TokenKind::Colon)?;
            let ty = self.parse_type()?;

            let param = ssa::Variable { name, ty, version };
            params.push(param);
            if !self.is_peek_a(TokenKind::Comma) {
                break;
            }
            self.consume(TokenKind::Comma)?;
            version += 1;
        }
        self.consume(TokenKind::RightParen)?;
        Ok(params)
    }

    fn parse_function_block(&mut self) -> Result<Vec<ssa::BasicBlock>, BitBoxError> {
        let mut blocks = vec![];
        self.consume(TokenKind::LeftBrace)?;
        while !self.end_of_stream() && !self.is_peek_a(TokenKind::RightBrace) {
            let block = self.parse_basic_block()?;
            blocks.push(block);
        }
        self.consume(TokenKind::RightBrace)?;
        Ok(blocks)
    }

    fn parse_basic_block(&mut self) -> Result<ssa::BasicBlock, BitBoxError> {
        let mut instructions = vec![];

        while !self.end_of_stream() && !self.is_peek_a(TokenKind::RightBrace) {
            let Some(instruction) = self.parse_instruction()? else {
                break;
            };
            instructions.push(instruction);
        }

        Ok(ssa::BasicBlock {
            id: 0,
            instructions,
            // TODO: Add predecessors and successors
            successors: vec![],
            predecessors: vec![],
        })
    }

    fn parse_instruction(&mut self) -> Result<Option<ssa::Instruction>, BitBoxError> {
        let tok = self.next()?;
        let Token {
            kind: TokenKind::Instruction(instruction),
            lexeme,
            span,
        } = tok
        else {
            return Err(BitBoxError::InvalidInstruction(tok));
        };
        match instruction {
            token::Instruction::Ret => self.parse_return(),
            token::Instruction::Add => self.parse_add(),
            token::Instruction::Sub => self.parse_sub(),
            token::Instruction::Call => self.parse_call(),
            token::Instruction::Phi => self.parse_phi(),
        }
    }

    fn parse_arguments(&mut self) -> Result<Vec<ssa::Operand>, BitBoxError> {
        self.consume(TokenKind::LeftParen)?;
        let mut args = vec![];
        while !self.end_of_stream() && !self.is_peek_a(TokenKind::RightParen) {
            let arg = self.parse_operand()?;
            args.push(arg);
            if !self.is_peek_a(TokenKind::Comma) {
                break;
            }
            self.consume(TokenKind::Comma)?;
        }
        self.consume(TokenKind::RightParen)?;
        Ok(args)
    }

    fn parse_operand(&mut self) -> Result<ssa::Operand, BitBoxError> {
        if self.is_peek_a(TokenKind::Number) {
            let tok = self.consume(TokenKind::Number)?;
            return Ok(ssa::Operand::Constant(tok));
        }
        let tok = self.consume(TokenKind::Identifier)?;
        Ok(ssa::Operand::Variable(tok))
    }

    fn parse_import_function_params(&mut self) -> Result<Vec<ssa::Type>, BitBoxError> {
        let mut params = vec![];
        self.consume(TokenKind::LeftParen)?;
        while !self.end_of_stream() {
            let ty = self.parse_type()?;
            params.push(ty);
            if !self.is_peek_a(TokenKind::Comma) {
                break;
            }
            self.consume(TokenKind::Comma)?;
        }
        self.consume(TokenKind::RightParen)?;
        Ok(params)
    }

    fn parse_import(&mut self) -> Result<ssa::Import, BitBoxError> {
        self.consume(TokenKind::Keyword(token::Keyword::Import))?;
        self.consume(TokenKind::Keyword(token::Keyword::Function))?;
        let module_name = self.consume(TokenKind::Identifier)?;
        self.consume(TokenKind::PathSeparator)?;
        let name = self.consume(TokenKind::Identifier)?;
        let params = self.parse_import_function_params()?;
        let return_type = self.parse_type()?;
        self.consume(TokenKind::Semicolon)?;
        Ok(ssa::Import::Function(ssa::FunctionSpec {
            module_name,
            name,
            params,
            return_type,
        }))
    }

    fn parse_constant_value(&mut self) -> Result<ssa::ConstantValue, BitBoxError> {
        let tok = self.next()?;
        match tok.kind {
            TokenKind::String => Ok(ssa::ConstantValue::String(tok)),
            TokenKind::Directive(directive) => match directive {
                token::Directive::Len => {
                    let value = self.consume(TokenKind::Identifier)?;
                    Ok(ssa::ConstantValue::Directive(ssa::Directive::Len(value)))
                }
            },
            _ => Err(BitBoxError::InvalidContantValue(tok)),
        }
    }

    fn parse_constant(&mut self) -> Result<ssa::Constant, BitBoxError> {
        self.consume(TokenKind::Keyword(token::Keyword::Const))?;
        let name = self.consume(TokenKind::Identifier)?;
        self.consume(TokenKind::Colon)?;
        let ty = self.parse_type()?;
        self.consume(TokenKind::Equals)?;
        let value = self.parse_constant_value()?;
        self.consume(TokenKind::Semicolon)?;
        Ok(ssa::Constant { name, ty, value })
    }

    fn parse_type(&mut self) -> Result<ssa::Type, BitBoxError> {
        let tok = self.next()?;
        match tok.kind {
            TokenKind::Identifier => tok
                .into_ssa_type()
                .map_err(|tok| BitBoxError::ExpectedType(tok)),
            TokenKind::Star => {
                let ty = self.parse_type()?;
                Ok(ssa::Type::Pointer(Box::new(ty)))
            }
            TokenKind::LeftBracket => {
                let count = self.consume(TokenKind::Number)?;
                self.consume(TokenKind::Semicolon)?;
                let ty = self.parse_type()?;
                self.consume(TokenKind::RightBracket)?;
                Ok(ssa::Type::Array(
                    count.lexeme.parse().unwrap(),
                    Box::new(ty),
                ))
            }
            _ => Err(BitBoxError::ExpectedType(tok)),
        }
    }

    fn parse_return(&mut self) -> Result<Option<ssa::Instruction>, BitBoxError> {
        let ty = self.parse_type()?;
        self.consume(TokenKind::Colon)?;
        let value = self.parse_operand()?;
        self.consume(TokenKind::Semicolon)?;
        Ok(Some(ssa::Instruction::Return(ty, value)))
    }

    fn parse_add(&mut self) -> Result<Option<ssa::Instruction>, BitBoxError> {
        let ty = self.parse_type()?;
        self.consume(TokenKind::Colon)?;
        let name = self.consume(TokenKind::Identifier)?;
        let des = ssa::Variable {
            name,
            ty,
            version: 0,
        };
        self.consume(TokenKind::Comma)?;
        let lhs = self.parse_operand()?;
        self.consume(TokenKind::Comma)?;
        let rhs = self.parse_operand()?;
        self.consume(TokenKind::Semicolon)?;
        Ok(Some(ssa::Instruction::Add(des, lhs, rhs)))
    }

    fn parse_sub(&mut self) -> Result<Option<ssa::Instruction>, BitBoxError> {
        let ty = self.parse_type()?;
        self.consume(TokenKind::Colon)?;
        let name = self.consume(TokenKind::Identifier)?;
        let des = ssa::Variable {
            name,
            ty,
            version: 0,
        };
        self.consume(TokenKind::Comma)?;
        let lhs = self.parse_operand()?;
        self.consume(TokenKind::Comma)?;
        let rhs = self.parse_operand()?;
        self.consume(TokenKind::Semicolon)?;
        Ok(Some(ssa::Instruction::Sub(des, lhs, rhs)))
    }

    fn parse_call(&mut self) -> Result<Option<ssa::Instruction>, BitBoxError> {
        let ty = self.parse_type()?;
        self.consume(TokenKind::Colon)?;
        let result_name = self.consume(TokenKind::Identifier)?;
        let des = ssa::Variable {
            name: result_name,
            ty,
            version: 0,
        };
        let name = self.consume(TokenKind::Identifier)?;
        let arguments = self.parse_arguments()?;
        self.consume(TokenKind::Semicolon)?;
        Ok(Some(ssa::Instruction::Call(des, name, arguments)))
    }

    fn parse_phi(&self) -> Result<Option<ssa::Instruction>, BitBoxError> {
        todo!("implement phi instruciton parser")
    }
}
