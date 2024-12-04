mod ast;
mod lexer;

fn main() {
    let src = r#"
function add(x: Number, y: Number) Number {
    z = x + y;
    return ;
}
"#;

    let tokens = lexer::lex(src);
    println!("{:#?}", tokens);
}
