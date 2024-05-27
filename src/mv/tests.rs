use super::*;
use crate::asm;
use anyhow::Result;

#[test]
fn test_load() -> Result<()> {
    let src = r#"
    .entry main
    main:
        load[u8] %0 100
"#;
    let program = asm::assemble(src)?;
    let mut mv = Mv::new(program)?;
    mv.run()?;
    assert_eq!(mv.get_regester(0), &100);
    for i in 1..Mv::REGESTER_COUNT {
        assert_eq!(mv.get_regester(i as u8), &0);
    }
    assert_eq!(mv.get_regester(0), &0);
    Ok(())
}
