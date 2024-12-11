#[cfg(test)]
mod test;
use crate::ast::{self, Identifier};
use crate::lexer::token::{Span, Token};
use crate::ssa::{self, IntoSsaType};
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

enum TopLevel {
    Function(ssa::Function),
    Import(ssa::Import),
    Constant(ssa::Constant),
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
        let mut imports = vec![];
        let mut functions = vec![];
        let mut constants = vec![];

        while self.stream.is_not_at_end() {
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
    fn parse_top_level(&mut self) -> Result<TopLevel, ParseError> {
        let visibility = self.parse_visibility();
        if self.peek_is_identifier("function") {
            Ok(TopLevel::Function(self.parse_function(visibility)?))
        } else if self.peek_is_identifier("import") {
            Ok(TopLevel::Import(self.parse_import()?))
        } else if self.peek_is_identifier("const") {
            Ok(TopLevel::Constant(self.parse_constant()?))
        } else {
            Err(ParseError::UnexpectedToken {
                expected: "top-level construct".to_string(),
                found: format!("{:?}", self.stream.peek_blind()),
                span: self.stream.current_span(),
            })
        }
    }

    fn parse_visibility(&mut self) -> ssa::Visibility {
        if self.peek_is_identifier("public") {
            self.consume::<ast::Identifier>().ok(); // Consume if present
            ssa::Visibility::Public
        } else {
            ssa::Visibility::Private
        }
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
                found: format!("{:?}", value),
                span: value.get_span(),
            }),
            None => Err(ParseError::UnexpectedEndOfStream),
        }
    }

    fn peek_is_identifier(&self, value: &str) -> bool {
        self.stream
            .peek::<ast::Identifier>()
            .map_or(false, |id| id.get_lexeme() == value)
    }

    fn consume_identifier(&mut self, value: &str) -> Result<ast::Identifier, ParseError> {
        let tok = self.consume::<Identifier>()?;
        if tok.lexeme != value {
            return Err(ParseError::UnexpectedToken {
                expected: value.to_string(),
                found: tok.lexeme.to_string(),
                span: tok.span,
            });
        }
        Ok(tok)
    }

    fn peek_is_builtin(&self, value: &str) -> bool {
        let Some(peek) = self.stream.peek::<ast::Builtin>().map(|i| i.get_lexeme()) else {
            return false;
        };

        value == peek
    }

    fn parse_function(&mut self, visibility: ssa::Visibility) -> Result<ssa::Function, ParseError> {
        self.consume_identifier("function")?;
        let func_name = self.consume::<ast::Identifier>()?;
        let params = self.parse_function_params()?;
        let return_type = self
            .consume::<ast::Identifier>()?
            .into_ssa_type()
            .map_err(|ident| ParseError::UnexpectedToken {
                expected: "Type".to_string(),
                found: ident.get_lexeme(),
                span: ident.get_span(),
            })?;
        let blocks = self.parse_function_block()?;

        Ok(ssa::Function {
            visibility,
            name: func_name.get_lexeme(),
            params,
            return_type,
            blocks,
        })
    }

    fn parse_function_params(&mut self) -> Result<Vec<ssa::Variable>, ParseError> {
        self.consume::<ast::LeftParen>()?;
        let mut params = vec![];

        let mut version = 0;
        while self.stream.is_not_at_end() && !self.stream.is_peek_a::<ast::RightParen>() {
            let name = self.consume::<ast::Identifier>()?;
            self.consume::<ast::Colon>()?;
            let ty = self.parse_type()?;

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
        while self.stream.is_not_at_end() && !self.stream.is_peek_a::<ast::RightBrace>() {
            let block = self.parse_basic_block()?;
            blocks.push(block);
        }
        self.consume::<ast::RightBrace>()?;
        Ok(blocks)
    }

    fn parse_basic_block(&mut self) -> Result<ssa::BasicBlock, ParseError> {
        let mut instructions = vec![];

        while self.stream.is_not_at_end() {
            if let Some(instruction) = self.parse_instruction()? {
                instructions.push(instruction);
            } else {
                break;
            }
        }

        Ok(ssa::BasicBlock {
            id: 0,
            instructions,
            // TODO: Add predecessors and successors
            successors: vec![],
            predecessors: vec![],
        })
    }

    fn parse_instruction(&mut self) -> Result<Option<ssa::Instruction>, ParseError> {
        if self.peek_is_builtin("@ret") {
            self.consume::<ast::Builtin>()?;
            let value = self.parse_operand()?;
            self.consume::<ast::Semicolon>()?;
            return Ok(Some(ssa::Instruction::Return(value)));
        }

        let name = self.consume::<ast::Identifier>()?;
        self.consume::<ast::Colon>()?;
        let ty = self.parse_type()?;
        self.consume::<ast::Equals>()?;

        if let Some(instruction) = self.parse_special_instruction(name.clone(), ty.clone())? {
            return Ok(Some(instruction));
        }

        Err(ParseError::UnexpectedToken {
            expected: "valid instruction".to_string(),
            found: name.lexeme,
            span: name.span,
        })
    }

    fn parse_special_instruction(
        &mut self,
        name: ast::Identifier,
        ty: ssa::Type,
    ) -> Result<Option<ssa::Instruction>, ParseError> {
        if self.peek_is_builtin("@add") {
            self.consume::<ast::Builtin>()?;
            let lhs = self.parse_operand()?;
            self.consume::<ast::Comma>()?;
            let rhs = self.parse_operand()?;
            self.consume::<ast::Semicolon>()?;
            return Ok(Some(ssa::Instruction::Add(
                ssa::Variable {
                    name,
                    ty,
                    version: 0,
                },
                lhs,
                rhs,
            )));
        }

        if self.peek_is_builtin("@sub") {
            self.consume::<ast::Builtin>()?;
            let lhs = self.parse_operand()?;
            self.consume::<ast::Comma>()?;
            let rhs = self.parse_operand()?;
            self.consume::<ast::Semicolon>()?;
            return Ok(Some(ssa::Instruction::Sub(
                ssa::Variable {
                    name,
                    ty,
                    version: 0,
                },
                lhs,
                rhs,
            )));
        }

        if self.peek_is_builtin("@call") {
            self.consume::<ast::Builtin>()?;
            let func_name = self.consume::<ast::Identifier>()?;
            let arguments = self.parse_arguments()?;
            self.consume::<ast::Semicolon>()?;
            return Ok(Some(ssa::Instruction::Call(
                ssa::Variable {
                    name,
                    ty,
                    version: 0,
                },
                func_name,
                arguments,
            )));
        }

        Ok(None)
    }

    fn parse_arguments(&mut self) -> Result<Vec<ssa::Operand>, ParseError> {
        self.consume::<ast::LeftParen>()?;
        let mut args = vec![];
        while self.stream.is_not_at_end() && !self.stream.is_peek_a::<ast::RightParen>() {
            args.push(self.parse_operand()?);
            if !self.stream.is_peek_a::<ast::Comma>() {
                break;
            }
            self.consume::<ast::Comma>()?;
        }
        self.consume::<ast::RightParen>()?;
        Ok(args)
    }

    fn parse_add_instruction(
        &mut self,
        name: ast::Identifier,
        ty: ssa::Type,
    ) -> Result<ssa::Instruction, ParseError> {
        self.consume::<ast::Builtin>()?;
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

    fn parse_operand(&mut self) -> Result<ssa::Operand, ParseError> {
        if self.stream.is_peek_a::<ast::Number>() {
            let tok = self.consume::<ast::Number>()?;
            return Ok(ssa::Operand::Constant(tok));
        }
        let tok = self.consume::<ast::Identifier>()?;
        Ok(ssa::Operand::Variable(tok))
    }

    fn parse_import_function_params(&mut self) -> Result<Vec<ssa::Type>, ParseError> {
        let mut params = vec![];
        self.consume::<ast::LeftParen>()?;
        while self.stream.is_not_at_end() {
            let ty = self.parse_type()?;
            params.push(ty);
            if !self.stream.is_peek_a::<ast::Comma>() {
                break;
            }
            self.consume::<ast::Comma>()?;
        }
        self.consume::<ast::RightParen>()?;
        Ok(params)
    }

    fn parse_import(&mut self) -> Result<ssa::Import, ParseError> {
        self.consume_identifier("import")?;
        let tok = self.consume::<ast::Identifier>()?;
        match tok.get_lexeme().as_str() {
            "function" => {
                let module_name = self.consume::<ast::Identifier>()?;
                self.consume::<ast::PathSeparator>()?;
                let name = self.consume::<ast::Identifier>()?;
                let params = self.parse_import_function_params()?;
                let return_type =
                    self.consume::<ast::Identifier>()?
                        .into_ssa_type()
                        .map_err(|ident| ParseError::UnexpectedToken {
                            expected: "Type".to_string(),
                            found: ident.get_lexeme(),
                            span: ident.get_span(),
                        })?;
                self.consume::<ast::Semicolon>()?;
                Ok(ssa::Import::Function(ssa::FunctionSpec {
                    module_name,
                    name,
                    params,
                    return_type,
                }))
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "function".to_string(),
                found: tok.get_lexeme(),
                span: tok.get_span(),
            }),
        }
    }

    fn parse_constant_value(&mut self) -> Result<ssa::ConstantValue, ParseError> {
        if self.stream.is_peek_a::<ast::BBString>() {
            let tok = self.consume::<ast::BBString>()?;
            return Ok(ssa::ConstantValue::String(tok.lexeme.clone()));
        } else if self.stream.is_peek_a::<ast::Directive>() {
            let tok = self.consume::<ast::Directive>()?;
            let value = self.consume::<ast::Identifier>()?;
            let directive = match tok.get_lexeme().as_str() {
                ".len" => ssa::Directive::Len(value),
                _ => unimplemented!("unimplemented directive: {}", tok.get_lexeme()),
            };
            return Ok(ssa::ConstantValue::Directive(directive));
        }
        unimplemented!("{:?}", self.stream.peek_blind());
    }

    fn parse_constant(&mut self) -> Result<ssa::Constant, ParseError> {
        self.consume_identifier("const")?;
        let name = self.consume::<ast::Identifier>()?;
        self.consume::<ast::Colon>()?;
        let ty = self.parse_type()?;
        self.consume::<ast::Equals>()?;
        let value = self.parse_constant_value()?;
        self.consume::<ast::Semicolon>()?;
        Ok(ssa::Constant { name, ty, value })
    }

    fn parse_type(&mut self) -> Result<ssa::Type, ParseError> {
        if self.stream.is_peek_a::<ast::Identifier>() {
            let tok = self.consume::<ast::Identifier>()?;
            return tok
                .into_ssa_type()
                .map_err(|ident| ParseError::UnexpectedToken {
                    expected: "Type".to_string(),
                    found: ident.get_lexeme(),
                    span: ident.get_span(),
                });
        } else if self.stream.is_peek_a::<ast::Star>() {
            self.consume::<ast::Star>()?;
            let ty = self.parse_type()?;
            Ok(ssa::Type::Pointer(Box::new(ty)))
        } else if self.stream.is_peek_a::<ast::LeftBracket>() {
            self.consume::<ast::LeftBracket>()?;
            let count = self.consume::<ast::Number>()?;
            self.consume::<ast::Semicolon>()?;
            let ty = self.parse_type()?;
            self.consume::<ast::RightBracket>()?;
            Ok(ssa::Type::Array(
                count.get_lexeme().parse().unwrap(),
                Box::new(ty),
            ))
        } else {
            unreachable!("parse_type {:?}", self.stream.peek_blind())
        }
    }
}
