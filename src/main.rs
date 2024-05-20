mod asm;
mod error;
mod instructions;
mod mv;
use anyhow::Result;

// TODO: PUSH
// TODO: POP
// TODO: INC
// TODO: EQ
// TODO: JNE
// TODO: SYSCALL for now we will do a print instruction

fn main() -> Result<()> {
    let src = r#"
    .entry main
    load[u32] %5 10
    main:
        load[u32] %0 1 ; a
        load[u32] %1 1 ; b
        load[u32] %2 46
    loop:
        push[u32] %1
        add[u32] %0 %1 %1
        pop[u32] %0
        inc[u32] %3
        ; eq[u32] %3 %2
        ; jne loop
        ; print %0
        ; SYSCALL
        ; load[u32] %0 0
        ; load[u32] %1 0
        hult
    "#;
    let program = asm::assemble(src)?;
    eprintln!("{:?}", program);
    mv::Mv::new(program)?.run()
}
