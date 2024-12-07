#![allow(dead_code)]
use crate::ast;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Variable {
    pub name: ast::Identifier,
    pub ty: ast::Identifier,
    pub version: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Instruction {
    Assign(Variable, Operand), // x: type = 1
    Add(Variable, Operand, Operand),
    Return(Operand),
    Phi(Variable, Vec<(Variable, usize)>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Operand {
    Variable(ast::Identifier),
    Constant(ast::Number),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BasicBlock {
    pub id: usize,
    pub instructions: Vec<Instruction>,
    pub successors: Vec<usize>,
    pub predecessors: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Visibility {
    Public,
    Private,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Function {
    pub visibility: Visibility,
    pub name: String,
    pub params: Vec<Variable>,
    pub return_type: ast::Identifier,
    pub blocks: Vec<BasicBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Program {
    pub functions: Vec<Function>,
}
