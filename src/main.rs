mod ast;
mod lexer;
mod parser;
mod ssa;
mod stream;

fn main() {
    let src = include_str!("../snapshots/basic.bitbox");
    let tokens = lexer::lex(src);
    let program = parser::Parser::new(tokens).parse();
    println!("{:#?}", program);
}
