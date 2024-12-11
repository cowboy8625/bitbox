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
    let program = match parser::Parser::new(tokens).parse() {
        Ok(program) => program,
        Err(err) => {
            print_error(err, &src, &filename);
            std::process::exit(1);
        }
    };
    let module = target::wasm::Emitter::new(program).with_no_main().emit();
    let bytes = module.to_bytes().unwrap();
    let (binary_name, _) = filename.split_once('.').unwrap();
    std::fs::write(format!("{}.wasm", binary_name), bytes).unwrap();
}

fn print_error(err: parser::ParseError, src: &str, filename: &str) {
    use parser::ParseError::*;
    match err {
        UnexpectedToken {
            expected,
            found,
            span,
        } => {
            let line = src[..span.start].chars().filter(|c| *c == '\n').count();
            let line = line + 1;
            let column = span.start - src[..span.start].rfind('\n').unwrap();
            eprintln!("{:?}", span);
            eprintln!(
                "{}:{}:{}: expected {}, found {}",
                filename, line, column, expected, found
            );

            let src_line = src.lines().nth(line - 1).unwrap();
            eprintln!("{line} | {}", src_line);
            let caret = " ".repeat(column) + "^";
            eprintln!("    {}", caret);
        }
        UnexpectedEndOfStream => eprintln!("Unexpected end of stream"),
    }
}
