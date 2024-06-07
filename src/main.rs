// TODO: Add inline strings
// TODO: Add inline char
// TODO: DATA section set string
use anyhow::Result;
use bitbox::prelude::*;

fn main() -> Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    let src = r#"
.entry main
main:

    hult
    "#;
    let program = assemble(src)?;
    let mut vm = Vm::new(program)?.with_args(args);
    vm.run()?;
    Ok(())
}
