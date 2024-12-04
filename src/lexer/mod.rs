mod lexer;
pub mod token;

#[cfg(test)]
mod test;

pub use lexer::Lexer;

pub fn lex(src: &str) -> Vec<Box<dyn token::Token>> {
    lexer::Lexer::new(src).collect()
}
