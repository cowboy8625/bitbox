mod asm;
mod error;
mod instructions;
mod utils;
mod vm;
use anyhow::Result;

// TODO: DATA section set string

fn main() -> Result<()> {
    let src = r#".entry main

main:
    load[u32] %0 6
    aloc[u32] %0

    load[u32] %0 0   ; pointer to string

    load[u8] %1 0x68 ; H
    store[u8] %0 %1
    inc[u8] %0

    load[u8] %1 0x65 ; e
    store[u8] %0 %1
    inc[u8] %0

    load[u8] %1 0x6c ; l
    store[u8] %0 %1
    inc[u8] %0

    load[u8] %1 0x6c ; l
    store[u8] %0 %1
    inc[u8] %0

    load[u8] %1 0x6f ; o
    store[u8] %0 %1
    inc[u8] %0

    load[u8] %1 0xa  ; \n
    store[u8] %0 %1

    load[u8] %0 0 ; syscall write
    load[u8] %1 0 ; pointer to string
    load[u8] %2 6 ; length of string
    load[u8] %3 0 ; static string
    syscall

    hult
    "#;
    let program = asm::assemble(src)?;
    let mut vm = vm::Vm::new(program)?;
    vm.run()?;
    // println!("stack: {:?}", vm.stack);
    // println!("regesters: {:?}", vm.regesters);
    // println!("heap: {:?}", vm.heap);
    Ok(())
}
