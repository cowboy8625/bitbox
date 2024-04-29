mod asm;
mod error;
mod instructions;
mod mv;
use anyhow::Result;

fn main() -> Result<()> {
    let src = r#"
    .entry main
    loadint %0 7
    loadint %1 10
    add %2 %0 %1
    hult
    "#;
    let program = asm::assemble(src)?;
    eprintln!("{:?}", program);
    mv::Mv::new(program)?.run()
}
