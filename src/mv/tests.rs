use super::*;
use crate::asm;
use anyhow::Result;

#[test]
fn test_load() -> Result<()> {
    let src = r#"
    .entry main
    main:
        load[u8] %0 100
        hult
"#;
    let program = asm::assemble(src)?;
    let mut mv = Mv::new(program)?;
    mv.run()?;
    assert_eq!(mv.get_regester(0), &100);
    for i in 1..Mv::REGESTER_COUNT {
        assert_eq!(mv.get_regester(i as u8), &0);
    }
    Ok(())
}

#[test]
fn test_or() -> Result<()> {
    let src = r#"
    .entry main
    main:
        load[u8] %0 1
        load[u8] %1 2
        or[u8] %0 %0 %1
        hult
"#;
    let program = asm::assemble(src)?;
    let mut mv = Mv::new(program)?;
    mv.run()?;
    assert_eq!(mv.get_regester(0), &3);
    assert_eq!(mv.get_regester(1), &2);
    for i in 2..Mv::REGESTER_COUNT {
        assert_eq!(mv.get_regester(i as u8), &0);
    }
    Ok(())
}
