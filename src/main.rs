use codegen::CodeGen;
use inkwell::context::Context;
use lexer::{Lexer, Token};
use parser::Parser;
use read_buffer::ReadBuffer;
use source::SourceCursor;
use std::fs::File;
use std::io::Read;
use std::process::Command;
use std::{env, fs};

pub mod codegen;
pub mod lexer;
pub mod parser;
pub mod read;
pub mod read_buffer;
pub mod source;

fn main() {
    // TODO: Use clap
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <file_path>", args[0]);
        std::process::exit(1);
    }

    let file_path = &args[1];
    let mut file = File::open(file_path).expect("Unable to open file");
    let mut input = String::new();
    file.read_to_string(&mut input)
        .expect("Unable to read file");

    let input = SourceCursor::new(&input);

    let tokens: Vec<Token> = Lexer::new(input).into_iter().collect();
    println!("----------Tokens------------");
    for t in &tokens {
        println!("{}", t);
    }

    let mut parser = Parser::new(ReadBuffer::new(tokens));
    let program = parser.parse().unwrap();
    println!("\n---------Statements----------");
    for st in &program {
        println!("{:?}", st);
    }

    let context = Context::create();
    let mut gen = CodeGen::new(&context);
    let module = gen.gen_code_for(program);
    println!("\n---------Generated LLVM IR----------");
    println!("{}", module.to_string());

    // Save to output.ll

    fs::create_dir_all("./out").expect("Could not create out directory");
    fs::write("./out/output.ll", &module.to_string()).expect("Unable to write file");

    // Compile the LLVM IR to an executable
    compile_to_executable();
}

fn compile_to_executable() {
    // Compile optimized LLVM IR to an object file
    let llc_output = Command::new("llc")
        .arg("./out/output.ll")
        .arg("-filetype=obj")
        .arg("-o")
        .arg("./out/output.o")
        .output()
        .expect("Failed to execute llc");

    if !llc_output.status.success() {
        eprintln!(
            "Error in llc: {}",
            String::from_utf8_lossy(&llc_output.stderr)
        );
        return;
    }

    // Link the object file to create an optimized executable
    let clang_output = Command::new("clang")
        .arg("./out/output.o")
        .arg("-O3") // Optimization level 3
        .arg("-o")
        .arg("./out/output")
        .output()
        .expect("Failed to execute clang");

    if !clang_output.status.success() {
        eprintln!(
            "Error in clang: {}",
            String::from_utf8_lossy(&clang_output.stderr)
        );
        return;
    }

    // Compile LLVM IR to an optimized LLVM IR file
    let opt_output = Command::new("opt")
        .arg("-O3") // Optimization level 3
        .arg("./out/output.ll")
        .arg("-o")
        .arg("./out/output_opt.ll")
        .output()
        .expect("Failed to execute opt");

    if !opt_output.status.success() {
        eprintln!(
            "Error in opt: {}",
            String::from_utf8_lossy(&opt_output.stderr)
        );
        return;
    }

    // Convert the optimized LLVM IR to human-readable assembly
    let llvm_dis_output = Command::new("llvm-dis")
        .arg("./out/output_opt.ll")
        .arg("-o")
        .arg("./out/output_opt.s")
        .output()
        .expect("Failed to execute llvm-dis");

    if !llvm_dis_output.status.success() {
        eprintln!(
            "Error in llvm-dis: {}",
            String::from_utf8_lossy(&llvm_dis_output.stderr)
        );
        return;
    }

    // Compile optimized LLVM IR to an object file
    let llc_output = Command::new("llc")
        .arg("./out/output_opt.ll")
        .arg("-filetype=obj")
        .arg("-o")
        .arg("./out/output_opt.o")
        .output()
        .expect("Failed to execute llc");

    if !llc_output.status.success() {
        eprintln!(
            "Error in llc: {}",
            String::from_utf8_lossy(&llc_output.stderr)
        );
        return;
    }

    // Link the object file to create an optimized executable
    let clang_output = Command::new("clang")
        .arg("./out/output_opt.o")
        .arg("-O3") // Optimization level 3
        .arg("-o")
        .arg("./out/output_opt")
        .output()
        .expect("Failed to execute clang");

    if !clang_output.status.success() {
        eprintln!(
            "Error in clang: {}",
            String::from_utf8_lossy(&clang_output.stderr)
        );
        return;
    }

    println!("Executable 'output' generated successfully!");
}
