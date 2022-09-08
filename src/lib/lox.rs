use std::env;
use std::fs;
use std::io::{self, BufRead, Write};

use crate::interpreter;
use crate::interpreter::Interpreter;
use crate::interpreter::RuntimeError;
use crate::parser::Expr;
use crate::parser::Parser;
use crate::scanner::{Scanner, Token, TokenType};

/// Effectively, the "Main" class. Handles the most top level operations on the Lox code.
pub struct Lox {
    had_error: bool,
    had_runtime_error: bool,
}

impl Lox {
    pub fn new() -> Self {
        Self { had_error: false, had_runtime_error: false }
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

        // Scan the source code into a list of Tokens
        let mut scanner = Scanner::new(src, self);
        let tokens = scanner.scan_tokens();

        // Parse the Tokens into a syntax tree
        let mut parser = Parser::new(tokens, self);
        let expr = parser.parse();

        // Exit if there were static errors
        if self.had_error {
            std::process::exit(65);
        }

        // Use the Tree Walk Interpreter to evaluate the expression
        let mut interpreter = Interpreter::new(self);
        interpreter.interpret(expr.clone().expect("Could not parse expression"));

        // Exit if there were Runtime errors
        if self.had_runtime_error {
            std::process::exit(70);
        }
    }

    /// Report a static error given a line and a msg (Used from Scanner)
    pub fn error(&mut self, line: usize, msg: String) {
        self.static_error(line, "".to_string(), msg);
    }

    /// Report a static error given a token and a Msg (called from Parser)
    pub fn error_token(&mut self, token: Token, msg: &str) {
        if token.token_type == TokenType::Eof {
            self.static_error(token.line, " at end".to_owned(), msg.to_owned());
        } else {
            self.static_error(
                token.line,
                format!(" at '{}'", token.lexeme),
                msg.to_owned(),
            );
        }
    }

    /// Internal method for reporting a static error
    fn static_error(&mut self, line: usize, location: String, msg: String) {
        eprintln!("[line {}] Error{}: {}", line, location, msg);
        self.had_error = true;
    }

    /// Report a runtime error (Called from the Interpreter)
    pub fn runtime_error(&mut self, error: RuntimeError) {
        eprintln!("{} [line {}]", error.msg, error.token.line);
        self.had_runtime_error = true;
    }
}
