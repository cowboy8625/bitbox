#[cfg(test)]
mod test;
use crate::ast;
use crate::lexer::token::{Span, Token};
use crate::ssa::{self, Visibility};
use crate::stream::TokenStream;

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken {
        expected: String,
        found: String,
        span: Span,
    },
    UnexpectedEndOfStream,
}

pub struct Parser {
    stream: TokenStream,
}

impl Parser {
    pub fn new(tokens: Vec<Box<dyn Token>>) -> Self {
        Self {
            stream: TokenStream::new(tokens),
        }
    }

    pub fn parse(&mut self) -> Result<ssa::Program, ParseError> {
        let mut functions = vec![];

        while self.stream.is_not_at_end() {
            let visibility = if self.peek_is_identifier("public") {
                self.consume::<ast::Identifier>()?;
                ssa::Visibility::Public
            } else {
                ssa::Visibility::Private
            };
            let function = self.parse_function(visibility)?;
            functions.push(function);
        }

        Ok(ssa::Program { functions })
    }

    fn consume<Expected>(&mut self) -> Result<Expected, ParseError>
    where
        Expected: Token + Clone,
    {
        match self.stream.next() {
            Some(value) if value.as_any().downcast_ref::<Expected>().is_some() => {
                Ok(value.as_any().downcast_ref::<Expected>().unwrap().clone())
            }
            Some(value) => Err(ParseError::UnexpectedToken {
                expected: std::any::type_name::<Expected>().to_string(),
                found: format!("{value:?}"),
                span: value.get_span(),
            }),
            None => Err(ParseError::UnexpectedEndOfStream),
        }
    }

    fn peek_is_identifier(&self, value: &str) -> bool {
        let Some(peek) = self
            .stream
            .peek::<ast::Identifier>()
            .map(|i| i.get_lexeme())
        else {
            return false;
        };

        value == peek
    }

    fn parse_function(&mut self, visibility: ssa::Visibility) -> Result<ssa::Function, ParseError> {
        self.consume::<ast::Function>()?;
        let func_name = self.consume::<ast::Identifier>()?;
        let arguments = self.parse_function_params()?;
        let return_type = self.consume::<ast::Identifier>()?;
        let blocks = self.parse_function_block()?;

        Ok(ssa::Function {
            visibility,
            name: func_name.get_lexeme(),
            arguments,
            return_type,
            blocks,
        })
    }

    fn parse_function_params(&mut self) -> Result<Vec<ssa::Variable>, ParseError> {
        self.consume::<ast::LeftParen>()?;
        let mut params = vec![];

        let mut version = 0;
        while self.stream.is_not_at_end() {
            let name = self.consume::<ast::Identifier>()?;
            self.consume::<ast::Colon>()?;
            let ty = self.consume::<ast::Identifier>()?;

            let param = ssa::Variable { name, ty, version };
            params.push(param);
            if !self.stream.is_peek_a::<ast::Comma>() {
                break;
            }
            self.consume::<ast::Comma>()?;
            version += 1;
        }
        self.consume::<ast::RightParen>()?;
        Ok(params)
    }

    fn parse_function_block(&mut self) -> Result<Vec<ssa::BasicBlock>, ParseError> {
        let mut blocks = vec![];
        self.consume::<ast::LeftBrace>()?;
        while self.stream.is_not_at_end() {
            let block = self.parse_basic_block()?;
            blocks.push(block);
        }
        self.consume::<ast::RightBrace>()?;
        Ok(blocks)
    }

    fn parse_basic_block(&mut self) -> Result<ssa::BasicBlock, ParseError> {
        let mut instructions = vec![];
        while self.stream.is_not_at_end() {
            if self.stream.is_peek_a::<ast::RightBrace>() {
                break;
            }

            if self.stream.is_peek_a::<ast::If>() {
                todo!();
                // continue;
            } else if self.stream.is_peek_a::<ast::Return>() {
                self.consume::<ast::Return>()?;
                let value = self.parse_operand()?;
                instructions.push(ssa::Instruction::Return(value));
                self.consume::<ast::Semicolon>()?;
                continue;
            }

            // z : i32 = add x, y;
            let name = self.consume::<ast::Identifier>()?;
            self.consume::<ast::Colon>()?;
            let ty = self.consume::<ast::Identifier>()?;
            self.consume::<ast::Equals>()?;

            if self.peek_is_identifier("add") {
                let instruction = self.parse_add_instruction(name, ty)?;
                instructions.push(instruction);
                continue;
            }
            todo!("unimplemented");
        }
        Ok(ssa::BasicBlock {
            id: 0,
            instructions,
            successors: vec![],
            predecessors: vec![],
        })
    }

    fn parse_add_instruction(
        &mut self,
        name: ast::Identifier,
        ty: ast::Identifier,
    ) -> Result<ssa::Instruction, ParseError> {
        self.consume::<ast::Identifier>()?;
        let var = ssa::Variable {
            name,
            ty,
            version: 0,
        };
        let lhs = self.parse_operand()?;
        self.consume::<ast::Comma>()?;
        let rhs = self.parse_operand()?;
        self.consume::<ast::Semicolon>()?;
        Ok(ssa::Instruction::Add(var, lhs, rhs))
    }

    fn parse_assignment(&mut self) -> Result<ssa::Instruction, ParseError> {
        todo!()
    }

    fn parse_binary_op(&mut self) -> Result<ssa::Instruction, ParseError> {
        todo!()
    }

    fn parse_operand(&mut self) -> Result<ssa::Operand, ParseError> {
        if self.stream.is_peek_a::<ast::Number>() {
            let tok = self.consume::<ast::Number>()?;
            return Ok(ssa::Operand::Constant(tok));
        }
        let tok = self.consume::<ast::Identifier>()?;
        Ok(ssa::Operand::Variable(tok))
    }
}
