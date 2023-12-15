mod img;
mod interpreter;
mod lexer;
mod parser;

use clap::{Parser, Subcommand};
use std::io::{BufRead, Write};

#[derive(Parser, Debug)]
#[command(
    author = "zetsuboii",
    version = "1.0.0",
    about = "Embed Brainf*ck programs in PNG images"
)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "Inject a Brainf*ck program into a PNG image")]
    Inject {
        #[arg(help = "PNG image to inject into")]
        image: String,

        #[arg(help = "Brainf*ck program")]
        program: String,

        #[arg(short, long, help = "Output file", default_value = "out.png")]
        output: String,
    },
    #[command(about="Execute a Brainf*ck program from a PNG image", aliases=["exec"])]
    Execute {
        #[arg(help = "PNG image to execute")]
        image: String,

        #[arg(short, long, help = "Verbose output", default_value = "false")]
        verbose: bool,
    },
    #[command(about = "Run a REPL (Read, Evaluate, Print, Loop) environment")]
    Repl {
        #[arg(short, long, help = "Verbose output", default_value = "false")]
        verbose: bool,
    },
}

fn main() {
    use interpreter::Interpreter;
    use lexer::Lexer;
    use parser::Parser;

    let args = Args::parse();

    match args.command {
        Commands::Inject {
            image,
            program,
            output,
        } => {
            let file_contents = match std::fs::read_to_string(program) {
                Ok(contents) => contents,
                Err(e) => {
                    eprintln!("Error while reading file: {}", e);
                    std::process::exit(1);
                }
            };

            let lexer = Lexer::new(file_contents);
            let tokens = match lexer.scan_tokens() {
                Ok(tokens) => tokens,
                Err(errors) => {
                    for (pos, msg) in errors {
                        eprintln!("Syntax error at position {pos}: {msg}");
                    }
                    std::process::exit(1);
                }
            };

            match img::write(&image, &output, tokens) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error while writing image: {}", e);
                    std::process::exit(1);
                }
            }

            println!("Wrote image to {}", output);
        }
        Commands::Execute { image, verbose } => {
            let tokens = match img::read(&image) {
                Ok(tokens) => tokens,
                Err(e) => {
                    eprintln!("Error while reading image: {}", e);
                    std::process::exit(1);
                }
            };

            if tokens.len() == 0 {
                println!("No program found");
                std::process::exit(1);
            }

            let parser = Parser::new(tokens);
            let mut ast: parser::Program = match parser.parse() {
                Ok(ast) => ast,
                Err(errors) => {
                    for (pos, msg) in errors {
                        eprintln!("Error at position {pos}: {msg}");
                    }
                    std::process::exit(1);
                }
            };

            let mut interpreter = Interpreter::new(vec![]);
            interpreter.interpret(&mut ast);
            interpreter.print_state(verbose);
        }
        Commands::Repl { verbose } => {
            run_repl(verbose);
        }
    }
}

/// Run a REPL (Read, Evaluate, Print, Loop) environment
fn run_repl(verbose: bool) {
    use lexer::Lexer;
    use parser::Parser;
    use interpreter::Interpreter;

    println!(":: Brainfreeze REPL ::");

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

        let lexer = Lexer::new(line);
        let tokens = match lexer.scan_tokens() {
            Ok(tokens) => tokens,
            Err(errors) => {
                for (pos, msg) in errors {
                    println!("Error at position {pos}: {msg}");
                }
                continue;
            },
        };

        let parser = Parser::new(tokens);
        let ast = parser.parse();

        let mut ast = match ast {
            Ok(ast) => ast,
            Err(errors) => {
                for (pos, msg) in errors {
                    println!("Error at position {pos}: {msg}");
                }
                continue;
            }
        };

        let mut interpreter = Interpreter::new(vec![]);
        interpreter.interpret(&mut ast);
        interpreter.print_state(verbose);
    }
}
