use cairo_m_common::Opcode;
use cairo_m_compiler_codegen::{InstructionBuilder, Operand};
use std::collections::HashMap;
use std::env;
use std::fs;
use wasmparser::{
    CompositeInnerType, ExternalKind, FuncType, FunctionBody, Operator, Parser, Payload,
};

pub struct CasmBuilder {
    pub instructions: Vec<InstructionBuilder>,
    pub labels: HashMap<String, usize>,
    pub stack: Vec<u32>,
    pub fp_offset: i32,
    pub function_names: HashMap<u32, String>, // Map function index to name
    pub function_types: Vec<u32>,             // Map function index to type index
    pub types: Vec<FuncType>,                 // Store function type definitions
}

impl CasmBuilder {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            labels: HashMap::new(),
            stack: Vec::new(),
            fp_offset: 0,
            function_names: HashMap::new(),
            function_types: Vec::new(),
            types: Vec::new(),
        }
    }

    pub fn build_file(&mut self, wasm_file: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        let mut function_index = 0;
        let parser = Parser::new(0);

        for payload in parser.parse_all(wasm_file) {
            match payload? {
                Payload::TypeSection(type_section) => {
                    for rec_group in type_section {
                        let rec_group = rec_group?;
                        for sub_type in rec_group.types() {
                            // Extract FuncType from SubType
                            if let CompositeInnerType::Func(func_type) =
                                &sub_type.composite_type.inner
                            {
                                self.types.push(func_type.clone());
                            }
                        }
                    }
                }
                Payload::FunctionSection(function_section) => {
                    for type_index in function_section {
                        let type_index = type_index?;
                        self.function_types.push(type_index);
                    }
                }
                Payload::ExportSection(export_section) => {
                    for export in export_section {
                        let export = export?;
                        match export.kind {
                            ExternalKind::Func => {
                                // Store the function name mapping
                                self.function_names
                                    .insert(export.index, export.name.to_string());
                            }
                            ExternalKind::Memory => {
                                // Skip memory exports
                            }
                            _ => {}
                        }
                    }
                }
                Payload::CodeSectionEntry(function_body) => {
                    // Get the proper function name, or use default
                    let function_name = self
                        .function_names
                        .get(&function_index)
                        .cloned()
                        .unwrap_or_else(|| format!("func_{}", function_index));

                    self.generate_function(function_name, function_body.clone(), function_index)?;

                    function_index += 1;
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub fn print_module(&self) {
        println!("=== WebAssembly Module Analysis ===");

        // Print type information
        println!("\n=== Function Types ({}) ===", self.types.len());
        for (i, func_type) in self.types.iter().enumerate() {
            let params: Vec<String> = func_type
                .params()
                .iter()
                .map(|p| format!("{:?}", p))
                .collect();
            let results: Vec<String> = func_type
                .results()
                .iter()
                .map(|r| format!("{:?}", r))
                .collect();
            println!(
                "  Type {}: ({}) -> ({})",
                i,
                params.join(", "),
                results.join(", ")
            );
        }

        // Print function information
        println!("\n=== Functions ({}) ===", self.function_types.len());
        for (func_idx, &type_idx) in self.function_types.iter().enumerate() {
            let name = self
                .function_names
                .get(&(func_idx as u32))
                .map(|s| s.as_str())
                .unwrap_or("unnamed");

            let param_count = if (type_idx as usize) < self.types.len() {
                self.types[type_idx as usize].params().len()
            } else {
                0
            };

            println!(
                "  Function {}: {} (type: {}, params: {})",
                func_idx, name, type_idx, param_count
            );
        }

        // Print exported functions
        println!("\n=== Exported Functions ===");
        for (index, name) in &self.function_names {
            println!("  {}: {}", name, index);
        }

        // Print generated CASM instructions
        println!(
            "\n=== Generated CASM Instructions ({}) ===",
            self.instructions.len()
        );
        for (i, instruction) in self.instructions.iter().enumerate() {
            println!("  [{}]: {}", i, self.format_instruction(instruction));
        }

        // Print function labels
        println!("\n=== Function Labels ===");
        for (label, addr) in &self.labels {
            println!("  {} -> instruction [{}]", label, addr);
        }
    }

    pub fn push_instruction(&mut self, instruction: InstructionBuilder) {
        self.instructions.push(instruction);
    }

    pub fn push_label(&mut self, label: String) {
        self.labels.insert(label, self.instructions.len());
    }

    pub fn local_get(&mut self, local_index: i32, param_count: u32) {
        let casm_offset = if (local_index as u32) < param_count {
            // Parameter access: WASM local 0,1,2... -> CASM fp-4, fp-5, fp-6...
            -4 - local_index
        } else {
            // Local variable access: use original local index for locals beyond parameters
            local_index - param_count as i32
        };

        self.push_instruction(
            InstructionBuilder::new(Opcode::StoreDerefFp as u32)
                .with_off0(casm_offset)
                .with_off2(self.fp_offset),
        );
        self.fp_offset += 1;
    }

    pub fn local_set(&mut self, local_index: i32, param_count: u32) {
        let casm_offset = if (local_index as u32) < param_count {
            // Parameter access: WASM local 0,1,2... -> CASM fp-4, fp-5, fp-6...
            -4 - local_index
        } else {
            // Local variable access: use original local index for locals beyond parameters
            local_index - param_count as i32
        };

        self.push_instruction(
            InstructionBuilder::new(Opcode::StoreDerefFp as u32)
                .with_off0(self.fp_offset - 1)
                .with_off2(casm_offset),
        );
        self.fp_offset -= 1;
    }

    pub fn local_tee(&mut self, local_index: i32, param_count: u32) {
        self.local_set(local_index, param_count);
        self.fp_offset += 1;
    }

    pub fn i32_add(&mut self) {
        self.push_instruction(
            InstructionBuilder::new(Opcode::StoreAddFpFp as u32)
                .with_off0(self.fp_offset - 2)
                .with_off1(self.fp_offset - 1)
                .with_off2(self.fp_offset - 2),
        );
        self.fp_offset -= 1;
    }

    pub fn i32_sub(&mut self) {
        self.push_instruction(
            InstructionBuilder::new(Opcode::StoreSubFpFp as u32)
                .with_off0(self.fp_offset - 2)
                .with_off1(self.fp_offset - 1)
                .with_off2(self.fp_offset - 2),
        );
        self.fp_offset -= 1;
    }

    pub fn i32_mul(&mut self) {
        self.push_instruction(
            InstructionBuilder::new(Opcode::StoreMulFpFp as u32)
                .with_off0(self.fp_offset - 2)
                .with_off1(self.fp_offset - 1)
                .with_off2(self.fp_offset - 2),
        );
        self.fp_offset -= 1;
    }

    pub fn i32_const(&mut self, value: i32) {
        self.push_instruction(
            InstructionBuilder::new(Opcode::StoreImm as u32)
                .with_off2(0)
                .with_imm(value),
        );
        self.fp_offset += 1;
    }

    pub fn call(&mut self, function_index: i32) {
        let label = self
            .function_names
            .get(&(function_index as u32))
            .cloned()
            .unwrap_or_else(|| format!("func_{}", function_index));

        self.push_instruction(InstructionBuilder::new(Opcode::CallAbsImm as u32).with_label(label));
        let returns_value = self.types[self.function_types[function_index as usize] as usize]
            .results()
            .len();
        if returns_value > 1 {
            panic!("Function {} returns multiple values", function_index);
        }
        self.fp_offset += returns_value as i32;
    }

    pub fn return_op(&mut self) {
        self.push_instruction(InstructionBuilder::new(Opcode::Ret as u32));
    }

    pub fn generate_function(
        &mut self,
        function_name: String,
        function_body: FunctionBody,
        function_index: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.labels
            .insert(function_name.clone(), self.instructions.len());

        // Get parameter count and return type info for this function
        let (param_count, has_return_value) =
            if (function_index as usize) < self.function_types.len() {
                let type_index = self.function_types[function_index as usize];
                if (type_index as usize) < self.types.len() {
                    let func_type = &self.types[type_index as usize];
                    (
                        func_type.params().len() as u32,
                        !func_type.results().is_empty(),
                    )
                } else {
                    (0, false)
                }
            } else {
                (0, false)
            };

        let locals_reader = function_body.get_locals_reader()?;
        let local_count = locals_reader.get_count() as i32;

        // Frame pointer starts at local count (accounting for locals beyond parameters)
        self.fp_offset = local_count;

        let operators_reader = function_body.get_operators_reader()?;

        for operator in operators_reader {
            match operator.clone()? {
                Operator::I32Add => self.i32_add(),
                Operator::I32Sub => self.i32_sub(),
                Operator::I32Mul => self.i32_mul(),
                Operator::I32Const { value } => self.i32_const(value),
                Operator::LocalGet { local_index } => {
                    self.local_get(local_index as i32, param_count)
                }
                Operator::LocalSet { local_index } => {
                    self.local_set(local_index as i32, param_count)
                }
                Operator::LocalTee { local_index } => {
                    self.local_tee(local_index as i32, param_count)
                }
                Operator::Call { function_index } => self.call(function_index as i32),
                Operator::End => {}
                Operator::Return => self.return_op(),
                _ => {
                    // Unsupported operators are silently ignored
                    println!("Unsupported operator: {:?}", operator);
                }
            }
        }

        if has_return_value {
            // Move the top of stack (fp_offset - 1) to the return value location (fp-3)
            self.push_instruction(
                InstructionBuilder::new(Opcode::StoreDerefFp as u32)
                    .with_off0(self.fp_offset - 1)
                    .with_off2(-3),
            );
            self.fp_offset -= 1;
        }
        self.push_instruction(InstructionBuilder::new(Opcode::Ret as u32));

        Ok(())
    }

    fn format_instruction(&self, instruction: &InstructionBuilder) -> String {
        use cairo_m_common::Opcode;

        let off0 = instruction.off0.unwrap_or(0);
        let off1 = instruction.off1.unwrap_or(0);
        let off2 = instruction.off2.unwrap_or(0);
        let operand = instruction.operand.clone();

        match instruction.opcode {
            op if op == Opcode::StoreAddFpFp as u32 => {
                format!("[fp + {}] = [fp + {}] + [fp + {}]", off2, off0, off1)
            }
            op if op == Opcode::StoreSubFpFp as u32 => {
                format!("[fp + {}] = [fp + {}] - [fp + {}]", off2, off0, off1)
            }
            op if op == Opcode::StoreMulFpFp as u32 => {
                format!("[fp + {}] = [fp + {}] * [fp + {}]", off2, off0, off1)
            }
            op if op == Opcode::StoreImm as u32 => {
                if let Some(Operand::Literal(imm)) = operand {
                    format!("[fp + {}] = {}", off2, imm)
                } else {
                    "error".to_string()
                }
            }
            op if op == Opcode::StoreDerefFp as u32 => {
                format!("[fp + {}] = [fp + {}]", off2, off0)
            }
            op if op == Opcode::CallAbsImm as u32 => {
                // Check if we have a function name stored for this instruction
                if let Some(Operand::Label(label)) = operand {
                    format!("call {}", label)
                } else {
                    "error".to_string()
                }
            }
            op if op == Opcode::JmpRelImm as u32 => {
                format!("jmp rel {}", off0)
            }
            op if op == Opcode::Ret as u32 => "ret".to_string(),
            _ => {
                format!(
                    "opcode_{} (off0:{} off1:{} off2:{})",
                    instruction.opcode, off0, off1, off2
                )
            }
        }
    }
}
