use super::{
    module::Module,
    opcode::Instruction,
    section::{
        _type::{FunctionType, ValueType},
        code::Block,
        export::{ExportEntry, ExportType},
        global::GlobalEntry,
        memory::Page,
    },
};
use crate::error::BitBoxError;

use super::ToDataType;
use crate::{ssa, target::wasm::section::global::Intializer};

#[derive(Debug)]
pub struct Emitter {
    module: Module,
    program: ssa::Program,
    no_main: bool,
}

impl Emitter {
    pub fn new(program: ssa::Program) -> Self {
        Self {
            module: Module::default(),
            program,
            no_main: false,
        }
    }

    pub fn with_no_main(mut self) -> Self {
        self.no_main = true;
        self
    }

    fn compile_instruction(
        &mut self,
        wasm_block: &mut Block,
        instruction: &ssa::Instruction,
        params: &[ssa::Variable],
    ) -> Result<(), BitBoxError> {
        match instruction {
            ssa::Instruction::Assign(_variable, _operand) => todo!(),
            ssa::Instruction::Add(variable, lhs, rhs) => {
                let Ok(data_type) = variable.ty.to_data_type() else {
                    panic!("Unknown Type {:?}", variable.ty);
                };
                wasm_block.push_local(&variable.name.lexeme, data_type);
                self.compile_operand(wasm_block, lhs, params)?;
                self.compile_operand(wasm_block, rhs, params)?;
                wasm_block.push(Instruction::I32Add);
                let Some(index) = wasm_block.get_local_index(&variable.name.lexeme, params.len())
                else {
                    panic!("Unknown Variable {:?}", variable);
                };
                wasm_block.push(Instruction::LocalSet(index as u32));
            }
            ssa::Instruction::Sub(variable, lhs, rhs) => {
                let Ok(data_type) = variable.ty.to_data_type() else {
                    panic!("Unknown Type {:?}", variable.ty);
                };
                wasm_block.push_local(&variable.name.lexeme, data_type);
                self.compile_operand(wasm_block, lhs, params)?;
                self.compile_operand(wasm_block, rhs, params)?;
                wasm_block.push(Instruction::I32Sub);
                let Some(index) = wasm_block.get_local_index(&variable.name.lexeme, params.len())
                else {
                    panic!("Unknown Variable {:?}", variable);
                };
                wasm_block.push(Instruction::LocalSet(index as u32));
            }
            ssa::Instruction::Return(_, operand) => {
                self.compile_operand(wasm_block, operand, params)?;
                wasm_block.push(Instruction::Return);
            }
            ssa::Instruction::Phi(_variable, _vec) => todo!(),
            ssa::Instruction::Call(variable, name, arguments) => {
                let Ok(data_type) = variable.ty.to_data_type() else {
                    panic!("Unknown Type {:?}", variable.ty);
                };
                wasm_block.push_local(&variable.name.lexeme, data_type);
                for argument in arguments.iter() {
                    self.compile_operand(wasm_block, argument, params)?;
                }
                let Some(id) = self.module.get_function_id(&name.lexeme) else {
                    panic!("Unknown Function {:?}", name);
                };
                wasm_block.push(Instruction::Call(id));
                let Some(index) = wasm_block.get_local_index(&variable.name.lexeme, params.len())
                else {
                    panic!("Unknown Variable {:?}", variable);
                };
                wasm_block.push(Instruction::LocalSet(index as u32));
            }
        }
        Ok(())
    }

    fn compile_operand(
        &mut self,
        wasm_block: &mut Block,
        operand: &ssa::Operand,
        params: &[ssa::Variable],
    ) -> Result<(), BitBoxError> {
        match operand {
            ssa::Operand::Variable(variable) => {
                if let Some(index) = params
                    .iter()
                    .position(|param| param.name.lexeme == *variable.lexeme)
                    .or(wasm_block.get_local_index(&variable.lexeme, params.len()))
                {
                    let instruction = Instruction::LocalGet(index as u32);
                    wasm_block.push(instruction);
                    return Ok(());
                } else if let Some(index) = self.module.get_global_index(&variable.lexeme) {
                    let instruction = Instruction::GlobalGet(index as u32);
                    wasm_block.push(instruction);
                    return Ok(());
                }
                panic!("Variable {:?} is not declare", variable);
            }
            ssa::Operand::Constant(number) => {
                // NOTE: unwrapping is ok here because we know the number is a number
                wasm_block.push(Instruction::I32Const(number.lexeme.parse().unwrap()));
            }
        }
        Ok(())
    }

    fn compile_basic_block(
        &mut self,
        expr: &[ssa::BasicBlock],
        params: &[ssa::Variable],
    ) -> Result<Block, BitBoxError> {
        let mut wasm_block = Block::default();
        for block in expr.iter() {
            for instruction in block.instructions.iter() {
                self.compile_instruction(&mut wasm_block, instruction, params)?;
            }
        }
        Ok(wasm_block)
    }

    fn compile_function_in_module(&mut self) -> Result<(), BitBoxError> {
        for func in self.program.functions.clone().into_iter() {
            let ssa::Function {
                visibility,
                name,
                params,
                return_type,
                blocks,
            } = func;

            let mut func_type = FunctionType::default();
            for var in params.iter() {
                let Ok(data_type) = var.ty.to_data_type() else {
                    panic!("Unknown Type {:?}", var.ty);
                };
                let value_type = ValueType::WithName(var.name.lexeme.to_string(), data_type);
                func_type = func_type.with_param(value_type);
            }

            func_type = if let Ok(return_type) = return_type.to_data_type() {
                func_type.with_result(return_type)
            } else {
                func_type
            };

            let block = self.compile_basic_block(&blocks, &params)?;

            self.module.add_function(&name, func_type, block);

            if let ssa::Visibility::Public = visibility {
                let Some(idx) = self.module.get_function_id(&name) else {
                    panic!("Unknown Function {:?}", name);
                };
                self.module
                    .export(ExportEntry::new(&name, ExportType::Func, idx as u32));
            }
        }
        Ok(())
    }

    pub fn compile_import_in_module(&mut self) -> Result<(), BitBoxError> {
        for import in self.program.imports.iter() {
            match import {
                ssa::Import::Function(spec) => {
                    let ssa::FunctionSpec {
                        module_name,
                        name,
                        params,
                        return_type,
                    } = spec;
                    let func = params
                        .into_iter()
                        .fold(FunctionType::default(), |acc, param| {
                            let Ok(data_type) = param.to_data_type() else {
                                panic!("Unknown Type {:?}", param);
                            };
                            acc.with_param(ValueType::Data(data_type))
                        });

                    let func = if let Ok(return_type) = return_type.to_data_type() {
                        func.with_result(return_type)
                    } else {
                        func
                    };
                    self.module.import(&module_name.lexeme, &name.lexeme, func);
                }
            }
        }
        Ok(())
    }

    pub fn compile_constant_in_module(&mut self) -> Result<(), BitBoxError> {
        for constant in self.program.constants.iter() {
            let ssa::Constant { name, ty: _, value } = constant;
            match value {
                ssa::ConstantValue::String(string) => {
                    let ptr = self.module.add_string(&name.lexeme, string);
                    let entry = GlobalEntry::new_i32(&name.lexeme, false, ptr);
                    self.module.add_global(entry);
                }
                ssa::ConstantValue::Directive(directive) => match directive {
                    ssa::Directive::Len(identifier) => {
                        let Some((_, entry)) = self.module.get_global(&identifier.lexeme) else {
                            panic!("Unknown Variable while compiling constant {:?}", identifier);
                        };
                        let Intializer::I32Const(id) = entry.intializer else {
                            panic!("Not I32Const while compiling constant {:?}", identifier);
                        };
                        let Some(segment) = self.module.get_data_segment_by_id(id as usize) else {
                            panic!("Not I32Const while compiling constant {:?}", identifier);
                        };

                        if segment.name != identifier.lexeme {
                            panic!("Not I32Const while compiling constant {:?}", identifier);
                        }

                        let entry =
                            GlobalEntry::new_i32(&name.lexeme, false, segment.data.len() as i32);
                        self.module.add_global(entry);
                    }
                },
            }
        }
        Ok(())
    }

    pub fn emit(mut self) -> Result<Module, BitBoxError> {
        self.module.add_memory(Page::WithNoMinimun(1));
        self.module
            .export(ExportEntry::new("memory", ExportType::Memory, 0));

        self.compile_import_in_module()?;
        self.compile_constant_in_module()?;
        self.compile_function_in_module()?;

        Ok(self.module)
    }
}
