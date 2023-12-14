#![allow(dead_code)]
mod img;
mod interpreter;
mod lexer;
mod parser;

use std::io::{BufRead, Write};

use img::*;
use interpreter::*;
use lexer::*;
use parser::*;

fn main() {
    let image_path = std::env::args().nth(1).expect("no file given");

    if let Some(program_path) = std::env::args().nth(2) {
        let contents = std::fs::read_to_string(program_path).expect("read file");
        let lexer = Lexer::new(contents);
        let tokens = lexer.scan_tokens().unwrap();

        encode_image(&image_path, tokens).unwrap();
    } else {
        let tokens = parse_image(image_path.as_ref()).unwrap();
        if tokens.len() == 0 {
            println!("No program found");
            std::process::exit(1);
        }
        let parser = Parser::new(tokens);
        let mut ast: Program = parser.parse().unwrap();
        let mut interpreter = Interpreter::new(vec![]);
        interpreter.interpret(&mut ast);

        let output = String::from_utf8(interpreter.state.output.clone()).unwrap();

        println!("Memory         :\t {:?}", interpreter.state.memory);
        println!("Pointer        :\t {:?}", interpreter.state.pointer);
        println!("Input          :\t {:?}", interpreter.state.input);
        println!("Output         :\t {:?}", interpreter.state.output);
        println!("Output (UTF-8) :\t {:?}", output);
    }

    // let tokens = parse_image(image_path.as_ref()).unwrap();
    // println!("{:?}", tokens);

    // println!(">> Brainfreeze <<");
    // // Skip the program name and collect arguments
    // let args: Vec<String> = std::env::args().skip(1).collect();
    // if args.len() > 1 {
    //     println!("Usage: brainfreeze <script>");
    //     std::process::exit(64);
    // } else if args.len() == 1 {
    //     run_file(args[0].as_ref());
    // } else {
    //     run_prompt();
    // }
}

/// Read a file and run the code inside it
fn run_file(path: &str) {
    let contents = std::fs::read_to_string(path).expect("[error] read file");
    if let Err(errors) = run(contents) {
        for (pos, msg) in errors {
            eprintln!("Error at line {pos}, {msg}");
            std::process::exit(65);
        }
    }
}

/// Run a REPL (Read, Evaluate, Print, Loop) environment
fn run_prompt() {
    let mut reader = std::io::BufReader::new(std::io::stdin());
    loop {
        let mut line = String::new();
        // Print prompt
        print!("> ");
        std::io::stdout().lock().flush().unwrap();
        // Read line
        reader.read_line(&mut line).expect("read line");
        // Trim line end
        line.pop();

        if line.len() == 0 {
            continue;
        }

        if let Err(errors) = run(line) {
            for (pos, msg) in errors {
                println!("Error at position {pos}: {msg}");
            }
        }
    }
}

/// Inner function to run the code
fn run(code: String) -> Result<(), Vec<LexError>> {
    let scanner = Lexer::new(code);
    let tokens = match scanner.scan_tokens() {
        Ok(tokens) => tokens,
        Err(errors) => return Err(errors),
    };

    let parser = Parser::new(tokens);
    let ast = parser.parse();

    let mut ast = match ast {
        Ok(ast) => ast,
        Err(errors) => {
            for (pos, msg) in errors {
                println!("Error at position {pos}: {msg}");
            }
            return Ok(());
        }
    };

    let mut interpreter = Interpreter::new(vec![]);
    interpreter.interpret(&mut ast);

    let output = String::from_utf8(interpreter.state.output.clone()).unwrap();

    println!("Memory         :\t {:?}", interpreter.state.memory);
    println!("Pointer        :\t {:?}", interpreter.state.pointer);
    println!("Input          :\t {:?}", interpreter.state.input);
    println!("Output         :\t {:?}", interpreter.state.output);
    println!("Output (UTF-8) :\t {:?}", output);

    Ok(())
}

// 2. Parsing
