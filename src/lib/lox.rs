use std::env;
use std::fs;
use std::io::{self, BufRead, Write};

use crate::error::StaticErrorReporter;
use crate::interpreter::Interpreter;
use crate::parser::Parser;
use crate::scanner::Scanner;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Mode {
    Repl,
    Script,
}

/// Effectively, the "Main" class. Handles the most top level operations on the Lox code.
pub struct Lox {
    interpreter: Interpreter,
    mode: Option<Mode>,
}

impl Lox {
    pub fn new() -> Self {
        let interpreter = Interpreter::new();
        Self {
            interpreter,
            mode: None,
        }
    }

    /// Handles parsing the command line arguments for the interpreter.
    pub fn lox_main(&mut self) {
        let args: Vec<String> = env::args().collect();

        if args.len() == 1 {
            // Running the executable with no arguments starts the repl
            self.run_prompt();
        } else if args.len() == 2 {
            // Running the executable with a single argument runs the provided filename as lox code
            self.run_file(&args[1]);
        } else {
            println!("Usage: jlox [script])");
            std::process::exit(64);
        }
    }

    /// Reads the contents of a file as a string and passes it to the run function.
    fn run_file(&mut self, filename: &str) {
        self.mode = Some(Mode::Script);
        let file_contents = fs::read_to_string(filename);
        match file_contents {
            Ok(code) => self.run(code),
            Err(e) => {
                eprintln!(
                    "Error attempting to run code in file {}. Associated error: {}",
                    filename, e
                );
            }
        }
    }

    /// Passes stdin to the run function line by line.
    fn run_prompt(&mut self) {
        self.mode = Some(Mode::Repl);
        let mut repl_active = true;

        while repl_active {
            // This print macro really ought to flush stdout
            print!("> ");
            io::stdout().flush().expect("Couldn't flush stdout");

            // Read in the line from stdin
            let stdin = io::stdin();
            let line = stdin.lock().lines().next().expect("No line provided");

            match line {
                // Run th eprovided line of code if there is one
                Ok(code) if !code.is_empty() => self.run(code),

                // Print an error if we get one while trying to read in the line
                Err(_) => {
                    eprintln!("Error reading line from terminal");
                }

                // This means is was an empty line, so we deactivate the repl
                _ => {
                    repl_active = false;
                }
            }
        }
    }

    /// Takes the code through each step of the lifecycle (scanning, parsing, ...)
    fn run(&mut self, src: String) {
        let debug_mode = false;

        let error_reporter = StaticErrorReporter::new();

        // Scan the source code into a list of Tokens
        let scanner = Scanner::new(src, error_reporter);
        let (tokens, error_reporter) = scanner.scan_tokens();
        if debug_mode {
            println!("Tokens: {:?}", tokens);
        }

        // Parse the Tokens into a syntax tree
        let parser = Parser::new(tokens, error_reporter);
        let (stmts, error_reporter) = parser.parse();
        if debug_mode {
            println!("Syntax Tree: {:?}", stmts.clone());
        }

        // Exit if there were static errors
        if error_reporter.had_error && self.mode == Some(Mode::Script) {
            std::process::exit(65);
        }

        // Use the Tree Walk Interpreter to evaluate the expression
        self.interpreter.interpret(stmts.clone());

        // Exit if there were Runtime errors
        if self.interpreter.error_reporter.had_error && self.mode == Some(Mode::Script) {
            std::process::exit(70);
        }
    }
}
