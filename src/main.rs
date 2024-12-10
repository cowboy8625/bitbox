use std::env::args;

mod ast;
mod lexer;
mod parser;
mod ssa;
mod stream;
mod target;

fn main() {
    let Some(filename) = args().nth(1) else {
        eprintln!("usage: bitbox <filename>");
        std::process::exit(1);
    };
    let src = std::fs::read_to_string(&filename).expect("failed to read file");
    let tokens = lexer::lex(&src);
    let program = parser::Parser::new(tokens).parse().unwrap();
    let module = target::wasm::Emitter::new(program).with_no_main().emit();
    let bytes = module.to_bytes().unwrap();
    let (binary_name, _) = filename.split_once('.').unwrap();
    std::fs::write(format!("{}.wasm", binary_name), bytes).unwrap();
}
