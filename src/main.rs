
mod lexer;
mod parser;

use std::io::{BufRead, Write};

use lexer::*;
use parser::*;

fn main() {
    println!(">> Brainfreeze <<");
    // Skip the program name and collect arguments
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() > 1 {
        println!("Usage: brainfreeze <script>");
        std::process::exit(64);
    } else if args.len() == 1 {
        run_file(args[0].as_ref());
    } else {
        run_prompt();
    }
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
fn run(code: String) -> Result<(), ScanError> {
    let scanner = Scanner::new(code);
    let tokens = match scanner.scan_tokens() {
        Ok(tokens) => tokens,
        Err(errors) => return Err(errors),
    };

    let parser = Parser::new(tokens);
    let ast = parser.parse();

    match ast {
        Ok(ast) => println!("{:#?}", &ast),
        Err(errors) => {
            for (pos, msg) in errors {
                println!("Error at position {pos}: {msg}");
            }
        }
    }

    Ok(())
}

// 2. Parsing
