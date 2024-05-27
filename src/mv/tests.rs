use super::*;
use crate::asm;
use anyhow::Result;

macro_rules! vm_test {
    ($name:ident, $src:expr, $(($reg:expr, $expected:expr) $(,)?)*) => {
        #[test]
        fn $name() -> Result<()> {
            let src = format!(
                "
                .entry main
                main:
                    {}
                    hult
                ",
                $src
            );
            let program = asm::assemble(&src)?;
            let mut mv = Mv::new(program)?;
            mv.run()?;
            let mut start = 0;
            $(
                assert_eq!(mv.get_regester($reg as u8), &$expected);
                start += 1;
            )*
            for i in start..Mv::REGESTER_COUNT {
                assert_eq!(mv.get_regester(i as u8), &0);
            }
            Ok(())
        }
    };
}

vm_test!(
    load,
    r#"
        load[u8] %0 100
        load[u32] %1 400
    "#,
    (Register::R0, 100),
    (Register::R1, 400),
);

vm_test!(
    push_pop,
    r#"
        load[u32] %0 10
        push[u32] %0
        pop[u32] %1
    "#,
    (Register::R0, 10),
    (Register::R1, 10),
);

vm_test!(
    add,
    r#"
        load[u32] %0 123
        load[u32] %1 321
        add[u32] %2 %0 %1
    "#,
    (Register::R0, 123),
    (Register::R1, 321),
    (Register::R2, 444),
);

vm_test!(
    sub,
    r#"
        load[u32] %0 124
        load[u32] %1 123
        sub[u32] %2 %0 %1
    "#,
    (Register::R0, 124),
    (Register::R1, 123),
    (Register::R2, 1),
);

vm_test!(
    inc,
    r#"
        load[u32] %0 0
        inc[u32] %0
        inc[u32] %0
        inc[u32] %0
    "#,
    (Register::R0, 3),
);

vm_test!(
    eq,
    r#"
        load[u32] %0 123
        load[u32] %1 321
        load[u32] %2 1000 ; this should over write when `eq` is called
        eq[u32] %2 %0 %1
    "#,
    (Register::R0, 123),
    (Register::R1, 321),
    (Register::R2, 0),
);

// TODO: Jne,
// TODO: Hult,
// TODO: PrintReg,

vm_test!(
    or,
    r#"
        load[u8] %0 1
        load[u8] %1 2
        or[u8] %0 %0 %1
    "#,
    (Register::R0, 3),
    (Register::R1, 2),
);

vm_test!(
    and,
    r#"
        load[u8] %0 255
        load[u8] %1 1
        and[u8] %0 %0 %1
    "#,
    (Register::R0, 1),
    (Register::R1, 1),
);
