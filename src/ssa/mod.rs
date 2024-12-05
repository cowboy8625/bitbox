#![allow(dead_code)]
use crate::ast;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SsaVariable {
    pub name: ast::Identifier,
    pub ty: ast::Identifier,
    pub version: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SsaInstruction {
    Assign(SsaVariable, Operand),                           // x: type = 1
    BinaryOp(SsaVariable, Operand, ast::Operator, Operand), // x: type = 1 + 2
    Return(Operand),
    Phi(SsaVariable, Vec<(SsaVariable, usize)>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Operand {
    Variable(ast::Identifier),
    Constant(ast::Number),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BasicBlock {
    pub id: usize,
    pub instructions: Vec<SsaInstruction>,
    pub successors: Vec<usize>,
    pub predecessors: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Function {
    pub name: String,
    pub arguments: Vec<SsaVariable>,
    pub return_type: ast::Identifier,
    pub blocks: Vec<BasicBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Program {
    pub functions: Vec<Function>,
}
