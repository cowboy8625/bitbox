// TODO: swapping from u32 to u64
mod asm;
mod error;
mod instructions;
mod utils;
mod vm;
use anyhow::Result;

// TODO: DATA section set string

fn main() -> Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    let src = r#"
.entry main
main:
    copy[u64] %31 %0 ; args length
    load[u64] %30 0  ; count
    load[u64] %0  1  ; length of string
    aloc[u64] %29 %0
    load[u8] %0  0xA  ; char `\n`
    store[u8] %29 %0

loop:
    ; get the ptr to string and length
    pop[u64] %5

    ; get ptr
    load[u64] %1 0b1111_1111_1111_1111_1111_1111_1111_1111
    and[u64] %1 %1 %5

    ; get length
    load[u64] %6 32
    shr[u64] %2 %5 %6 ; shift right by 32 bits

    ; write first arg
    load[u8] %0 0
    ; %1 ptr
    ; %2 length
    load[u8] %3 0
    syscall

    copy[u64] %1 %29 ; ptr
    load[u64] %2 1   ; length
    syscall

    inc[u64] %30
    jne %31 %30 loop

    hult
    "#;
    let program = asm::assemble(src)?;
    let mut vm = vm::Vm::new(program)?.with_args(args);
    vm.run()?;
    Ok(())
}
