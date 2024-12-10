use std::env::args;

mod ast;
mod lexer;
mod parser;
mod ssa;
mod stream;
mod target;

fn main() {
    let backup = include_str!("../snapshots/import_function.bitbox");
    let src = args().nth(1).unwrap_or(backup.to_string());
    let tokens = lexer::lex(&src);
    let program = parser::Parser::new(tokens).parse().unwrap();
    let module = target::wasm::Emitter::new(program).with_no_main().emit();
    let bytes = module.to_bytes().unwrap();
    std::fs::create_dir_all("junk").unwrap();
    std::fs::write("junk/test.wasm", bytes).unwrap();
}
