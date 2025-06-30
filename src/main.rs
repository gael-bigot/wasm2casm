mod transpiler;

use crate::transpiler::CasmBuilder;
use std::env;
use std::fs;
use wasmparser::{
    CompositeInnerType, ExternalKind, FuncType, FunctionBody, Operator, Parser, Payload,
};

pub fn disassemble(wasm_file: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let parser = Parser::new(0);
    let mut function_index = 0;

    for payload in parser.parse_all(wasm_file) {
        match payload? {
            Payload::CodeSectionEntry(function_body) => {
                println!("=== Function {} ===", function_index);
                let operators_reader = function_body.get_operators_reader()?;
                let mut instruction_index = 0;

                for operator in operators_reader {
                    println!("  [{}]: {:?}", instruction_index, operator?);
                    instruction_index += 1;
                }
                function_index += 1;
                println!();
            }
            _ => {}
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 || args.len() > 3 {
        eprintln!("Usage: {} [-r] <wasm_file>", args[0]);
        eprintln!("  -r: Read disassembled WASM without transpiling to CASM");
        eprintln!("Example: {} test.wasm", args[0]);
        eprintln!("Example: {} -r test.wasm", args[0]);
        std::process::exit(1);
    }

    let (read_only, filename) = if args.len() == 3 {
        if args[1] == "-r" {
            (true, &args[2])
        } else {
            eprintln!("Unknown option: {}", args[1]);
            std::process::exit(1);
        }
    } else {
        (false, &args[1])
    };

    let wasm_file = fs::read(filename)?;

    if read_only {
        println!("=== WASM Disassembly ===");
        disassemble(&wasm_file)?;
    } else {
        println!("=== WASM to CASM Transpilation ===");
        let mut code_generator = CasmBuilder::new();
        code_generator.build_file(&wasm_file)?;
        code_generator.print_module();
    }

    Ok(())
}
