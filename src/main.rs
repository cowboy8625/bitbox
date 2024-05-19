mod asm;
mod error;
mod instructions;
mod mv;
use anyhow::Result;

fn main() -> Result<()> {
    let src = r#"
    .entry main
    load[u32] %5 10
    main:
        load[u32] %0 7
        load[u32] %1 10
        add[u32] %2 %0 %1
        hult
    "#;
    let program = asm::assemble(src)?;
    eprintln!("{:?}", program);
    mv::Mv::new(program)?.run()
}
