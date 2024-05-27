mod asm;
mod error;
mod instructions;
mod mv;
mod utils;
use anyhow::Result;

// TODO: SYSCALL for now we will do a print instruction

fn main() -> Result<()> {
    let src = r#"
    .entry main
    main:
        load[u8] %0 1
        load[u8] %1 2
        or[u8] %0 %0 %1
        printreg[u8] %0
        hult
    "#;
    let program = asm::assemble(src)?;
    mv::Mv::new(program)?.run()
}
