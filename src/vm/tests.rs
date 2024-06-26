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
            let mut vm = Vm::new(program)?;
            vm.run()?;
            let mut start = 0;
            $(
                assert_eq!(vm.get_regester($reg as u8), &$expected);
                start += 1;
            )*
            for i in start..Vm::REGESTER_COUNT {
                assert_eq!(vm.get_regester(i as u8), &0);
            }
            Ok(())
        }
    };
}

vm_test!(
    load_imm,
    r#"
        load[u8] %0 100
        load[u32] %1 400
    "#,
    (Register::R0, 100),
    (Register::R1, 400),
);

vm_test!(
    copy_reg,
    r#"
        load[u8] %0 100
        copy[u8] %1 %0
    "#,
    (Register::R0, 100),
    (Register::R1, 100),
);

vm_test!(
    load_imm_hex,
    r#"
        load[u8] %0 0x6_4
    "#,
    (Register::R0, 0x64),
);

vm_test!(
    load_imm_bin,
    r#"
        load[u8] %0 0b0000_0010
    "#,
    (Register::R0, 2),
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
    div,
    r#"
        load[u32] %0 100
        load[u32] %1 5
        div[u32] %2 %0 %1
    "#,
    (Register::R0, 100), // lhs
    (Register::R1, 5),   // rhs
    (Register::R2, 20),  // result
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
    shift_right,
    r#"
        load[u8] %0 2
        load[u8] %1 1
        shr[u8] %2 %0 %1
    "#,
    (Register::R0, 2),
    (Register::R1, 1),
    (Register::R2, 1),
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

#[test]
fn call() -> Result<()> {
    let src = r#"
    .entry main
    my_add:
        add[u32] %0 %1 %0
        return

    main:
        load[u32] %0 123
        load[u32] %1 321
        call my_add
        hult
    "#;
    let program = asm::assemble(&src)?;
    let mut vm = Vm::new(program)?;
    vm.run()?;
    assert_eq!(vm.get_regester(0), &444);
    assert_eq!(vm.get_regester(1), &321);
    for i in 3..Vm::REGESTER_COUNT {
        assert_eq!(vm.get_regester(i as u8), &0);
    }
    Ok(())
}

#[test]
fn store() -> Result<()> {
    let src = r#"
        .entry main
        main:
            load[u8] %0 1   ; One Byte
            aloc[u8] %2 %0     ; Allocate 1 Byte
            load[u8] %1 100 ; Value
            store[u8] %2 %1 ; Store Pointer/Index Value
            hult
        "#;
    let program = asm::assemble(&src)?;
    let mut vm = Vm::new(program)?;
    vm.run()?;
    assert_eq!(vm.get_regester(0), &1);
    assert_eq!(vm.get_regester(1), &100);
    assert_eq!(vm.get_regester(2), &0);
    for i in 3..Vm::REGESTER_COUNT {
        assert_eq!(vm.get_regester(i as u8), &0);
    }
    assert_eq!(vm.heap.len(), 1);
    assert_eq!(vm.heap[0], 100);
    Ok(())
}

#[test]
fn aloc() -> Result<()> {
    let src = r#"
        .entry main
        main:
            load[u8] %0 1
            aloc[u8] %1 %0
            hult
        "#;
    let program = asm::assemble(&src)?;
    let mut vm = Vm::new(program)?;
    vm.run()?;
    assert_eq!(vm.get_regester(0), &1);
    assert_eq!(vm.get_regester(1), &0);
    for i in 2..Vm::REGESTER_COUNT {
        assert_eq!(vm.get_regester(i as u8), &0);
    }
    assert_eq!(vm.heap.len(), 1);
    Ok(())
}
