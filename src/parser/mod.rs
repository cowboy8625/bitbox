use crate::ast::{self, Identifier};
use crate::lexer::token::{Span, Token};
use crate::ssa;
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
            let function = self.parse_function()?;
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
                found: value.get_lexeme(),
                span: value.get_span(),
            }),
            None => Err(ParseError::UnexpectedEndOfStream),
        }
    }

    fn parse_function(&mut self) -> Result<ssa::Function, ParseError> {
        self.consume::<ast::Function>()?;
        let func_name = self.consume::<ast::Identifier>()?;
        let arguments = self.parse_function_params()?;
        let return_type = self.consume::<ast::Identifier>()?;
        let blocks = self.parse_function_block()?;
        eprintln!("Parsing function {}", func_name.get_lexeme());

        Ok(ssa::Function {
            name: func_name.get_lexeme(),
            arguments,
            return_type,
            blocks,
        })
    }

    fn parse_function_params(&mut self) -> Result<Vec<ssa::SsaVariable>, ParseError> {
        self.consume::<ast::LeftParen>()?;
        let mut params = vec![];

        let mut version = 0;
        while self.stream.is_not_at_end() {
            let name = self.consume::<ast::Identifier>()?;
            self.consume::<ast::Colon>()?;
            let ty = self.consume::<ast::Identifier>()?;

            let param = ssa::SsaVariable { name, ty, version };
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
                instructions.push(ssa::SsaInstruction::Return(value));
                self.consume::<ast::Semicolon>()?;
                continue;
            }

            let name = self.consume::<ast::Identifier>()?;
            self.consume::<ast::Colon>()?;
            let ty = self.consume::<ast::Identifier>()?;
            self.consume::<ast::Equals>()?;
            let lhs = self.parse_operand()?;
            if self.stream.is_peek_a::<ast::Semicolon>() {
                self.consume::<ast::Semicolon>()?;
                let variable = ssa::SsaVariable {
                    name,
                    ty,
                    version: 0,
                };
                instructions.push(ssa::SsaInstruction::Assign(variable, lhs));
                continue;
            }

            let variable = ssa::SsaVariable {
                name,
                ty,
                version: 0,
            };
            let operator = self.parse_operator()?;
            let rhs = self.parse_operand()?;
            instructions.push(ssa::SsaInstruction::BinaryOp(variable, lhs, operator, rhs));
            self.consume::<ast::Semicolon>()?;
        }
        Ok(ssa::BasicBlock {
            id: 0,
            instructions,
            successors: vec![],
            predecessors: vec![],
        })
    }

    fn parse_assignment(&mut self) -> Result<ssa::SsaInstruction, ParseError> {
        todo!()
    }

    fn parse_binary_op(&mut self) -> Result<ssa::SsaInstruction, ParseError> {
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

    fn parse_operator(&mut self) -> Result<ast::Operator, ParseError> {
        self.consume::<ast::Plus>().map(ast::Operator::Add)
    }
}
