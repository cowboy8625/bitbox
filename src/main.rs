use std::env::args;

mod error;
mod lexer;
mod parser;
mod ssa;
mod target;

fn main() {
    let Some(filename) = args().nth(1) else {
        eprintln!("usage: bitbox <filename>");
        std::process::exit(1);
    };
    let src = std::fs::read_to_string(&filename).expect("failed to read file");
    let tokens = lexer::lex(&src);
    let program = match parser::Parser::new(tokens).parse() {
        Ok(program) => program,
        Err(err) => {
            let formated_error = err.report(&filename, &src);
            eprintln!("{formated_error}");
            std::process::exit(1);
        }
    };
    let module = match target::wasm::Emitter::new(program).with_no_main().emit() {
        Ok(module) => module,
        Err(err) => {
            let formated_error = err.report(&filename, &src);
            eprintln!("{formated_error}");
            std::process::exit(1);
        }
    };
    let bytes = module.to_bytes().unwrap();
    let (binary_name, _) = filename.split_once('.').unwrap();
    std::fs::write(format!("{}.wasm", binary_name), bytes).unwrap();
}
