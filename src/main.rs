mod asm;
mod error;
mod instructions;
mod utils;
mod vm;
use anyhow::Result;

// TODO: SYSCALL for now we will do a print instruction

fn main() -> Result<()> {
    let src = r#"
    .entry main

    my_add:
        add[u32] %0 %0 %1
        return

    main:
        ; load[u32] %0 4
        ; aloc[u32] %0
        ; load[u32] %1 0
        ; load[u32] %2 100
        ; store[u32] %1 %2
        load[u32] %0 123
        load[u32] %1 321
        call my_add
        hult
    "#;
    let program = asm::assemble(src)?;
    let mut vm = vm::Vm::new(program)?;
    vm.run()?;
    Ok(())
}
