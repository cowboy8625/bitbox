#![allow(dead_code)]
use crate::lexer::token::{Token, TokenKind};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Unsigned(u8),
    Signed(u8),
    Float(u8),
    Pointer(Box<Type>),
    Array(usize, Box<Type>),
    Void,
}

pub trait IntoSsaType {
    type Error;
    fn into_ssa_type(&self) -> Result<Type, Self::Error>;
}

impl IntoSsaType for Token {
    type Error = Self;
    fn into_ssa_type<'a>(&'a self) -> Result<Type, Self::Error> {
        let parse_type = |input: &'a str| -> Option<(&'a str, &'a str)> {
            let (prefix, rest) = input.split_at(1);
            if prefix.chars().all(char::is_alphabetic) && rest.chars().all(char::is_numeric) {
                Some((prefix, rest))
            } else {
                None
            }
        };
        if self.lexeme.as_str() == "void" {
            return Ok(Type::Void);
        }
        let Some((prefix, number)) = parse_type(&self.lexeme) else {
            return Err(self.clone());
        };
        match prefix {
            "u" => Ok(Type::Unsigned(number.parse().unwrap())),
            "s" => Ok(Type::Signed(number.parse().unwrap())),
            "f" => Ok(Type::Float(number.parse().unwrap())),
            "*" => {
                let ty = Token {
                    lexeme: number.to_string(),
                    kind: TokenKind::Number,
                    span: 0..0,
                }
                .into_ssa_type()?;
                Ok(Type::Pointer(Box::new(ty)))
            }
            _ => Err(self.clone()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Variable {
    pub name: Token,
    pub ty: Type,
    pub version: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Instruction {
    Assign(Variable, Operand),
    Add(Variable, Operand, Operand),
    Sub(Variable, Operand, Operand),
    Return(Type, Operand),
    Call(Variable, Token, Vec<Operand>),
    Phi(Variable, Vec<(Variable, usize)>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Operand {
    Variable(Token),
    Constant(Token),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BasicBlock {
    pub id: usize,
    pub instructions: Vec<Instruction>,
    pub successors: Vec<usize>,
    pub predecessors: Vec<usize>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub enum Visibility {
    Public,
    #[default]
    Private,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Function {
    pub visibility: Visibility,
    pub name: String,
    pub params: Vec<Variable>,
    pub return_type: Type,
    pub blocks: Vec<BasicBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionSpec {
    pub module_name: Token,
    pub name: Token,
    pub params: Vec<Type>,
    pub return_type: Type,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Import {
    Function(FunctionSpec),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Directive {
    Len(Token),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConstantValue {
    String(Token),
    Directive(Directive),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Constant {
    pub name: Token,
    pub ty: Type,
    pub value: ConstantValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Program {
    pub imports: Vec<Import>,
    pub constants: Vec<Constant>,
    pub functions: Vec<Function>,
}
