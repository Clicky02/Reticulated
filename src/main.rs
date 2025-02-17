use lexer::{Lexer, Token};
use parser::Parser;
use read_buffer::ReadBuffer;
use source::SourceCursor;
use std::env;
use std::fs::File;
use std::io::Read;

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
    println!("\n---------Statements----------");
    for st in parser.parse().unwrap() {
        println!("{:?}", st);
    }
}
