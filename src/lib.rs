// pub mod asm;
// pub mod error;
// pub mod instructions;
// pub mod prelude;
// pub mod utils;
// pub mod vm;
//
// use anyhow::Result;
// use asm::{Item, Text};
// use instructions::{Data, Instruction, Label, Opcode, Register, Type};
//
// use std::collections::HashMap;
//
// #[derive(Debug)]
// pub struct Module {
//     name: String,
//     functions: HashMap<String, Function>,
// }
//
// impl Module {
//     pub fn new(name: impl Into<String>) -> Self {
//         Self {
//             name: name.into(),
//             functions: HashMap::new(),
//         }
//     }
//
//     pub fn function<F>(mut self, name: impl Into<String>, return_type: Type, func: F) -> Self
//     where
//         F: FnOnce(&mut Function),
//     {
//         let name = name.into();
//         let mut function = Function {
//             name: name.clone(),
//             return_type,
//             params: Vec::new(),
//             body: Body::new(),
//         };
//         func(&mut function);
//         self.functions.insert(name, function);
//         self
//     }
//
//     pub fn build(self) -> Result<Text> {
//         let mut texts = vec![];
//         for (name, function) in self.functions.into_iter() {
//             texts.push(Text {
//                 label: Some(Label {
//                     name,
//                     span: Span::default(),
//                     def: true,
//                 }),
//                 opcode: Instruction,
//                 span: Span,
//             });
//         }
//         Ok(text)
//     }
// }
//
// #[derive(Debug)]
// pub struct Function {
//     name: String,
//     return_type: Type,
//     params: Vec<Param>,
//     body: Body,
// }
//
// impl Function {
//     pub fn add_param(&mut self, name: impl Into<String>, ty: Type) {
//         self.params.push(Param {
//             name: name.into(),
//             ty,
//         });
//     }
//
//     pub fn get_param(&self, name: &str) -> Option<Register> {
//         self.params
//             .iter()
//             .enumerate()
//             .find(|(_, param)| param.name == name)
//             .and_then(|(i, _)| Register::try_from(i as u8).ok())
//     }
//
//     pub fn body<F>(&mut self, f: F)
//     where
//         F: FnOnce(&mut Body),
//     {
//         f(&mut self.body);
//     }
// }
//
// #[derive(Debug)]
// pub struct Param {
//     name: String,
//     ty: Type,
// }
//
// #[derive(Debug)]
// pub struct Body {
//     instructions: Vec<Instruction>,
// }
//
// impl Body {
//     fn new() -> Self {
//         Self {
//             instructions: Vec::new(),
//         }
//     }
//
//     fn add(&mut self, des: Register, lhs: Register, rhs: Register) -> Register {
//         self.instructions.push(Instruction {
//             opcode: Opcode::Add,
//             r#type: Type::U(32),
//             data: Data::Reg3(des, lhs, rhs),
//         });
//         des
//     }
//
//     fn ret(&mut self) {
//         self.instructions.push(Instruction {
//             opcode: Opcode::Return,
//             r#type: Type::Void,
//             data: Data::NoArgs,
//         });
//     }
// }
//
// #[test]
// fn testing_module() {
//     let module = Module::new("test").function("test_function", Type::U(32), |func| {
//         func.add_param("x", Type::U(32));
//         func.add_param("y", Type::U(32));
//         let Some(x) = func.get_param("x") else {
//             panic!("Missing parameter 'y'");
//         };
//         let Some(y) = func.get_param("y") else {
//             panic!("Missing parameter 'y'");
//         };
//         func.body(|body| {
//             body.add(x, x, y);
//             body.ret();
//         });
//     });
//
//     eprintln!("{:#?}", module);
//
//     assert!(false);
// }
