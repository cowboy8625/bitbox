// TODO: swapping from u32 to u64
mod asm;
mod error;
mod instructions;
mod utils;
mod vm;
use anyhow::Result;

// TODO: DATA section set string

fn main() -> Result<()> {
    let src = r#".entry main

.entry main
main:
    load[u64] %0  1 ; a = 1
    load[u64] %1  1 ; b = 1
    load[u64] %2 93 ; c = 46 (the number of iterations)
    load[u64] %3  2 ; d = 2 (to start counting from the third Fibonacci number)
loop:
    push[u64] %1    ; push b to stack
    add[u64] %1 %0 %1 ; b = a + b
    pop[u64] %0     ; a = old b (from stack)
    inc[u64] %3     ; d++
    jne %3 %2 loop  ; if d != c, jump to loop
    printreg[u64] %1 ; print b (the last computed Fibonacci number)
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
