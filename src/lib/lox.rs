use std::env;
use std::fs;
use std::io::{self, BufRead, Write};

use crate::scanner::Scanner;

pub struct Lox {
    had_error: bool,
}

impl Lox {

    pub fn new() -> Self {
        Self {
            had_error: false
        }
    }

    pub fn lox_main(&mut self, ) {
        let args: Vec<String> = env::args().collect();
        if args.len() == 1 {
            self.run_prompt();
        } else if args.len() == 2 {
            self.run_file(&args[1]);
        } else {
            println!("Usage: jlox [script])");
            std::process::exit(64);
        }
    }
    
    // Reads the contents of a file as a string and passes the file to the run function.
    fn run_file(&mut self, filename: &str) { 
        let file_contents = fs::read_to_string(filename);
        match file_contents {
            Ok(code) => self.run(code),
            Err(e) => {
                eprintln!("Error attempting to run code in file {}. Associated error: {}", filename, e);
            },
        }
    }
    
    // Passes stdin to the run function line by line.
    fn run_prompt(&mut self, ) {
    
        let mut repl_active = true;
    
        while repl_active {
            print!("> ");
            io::stdout().flush().expect("Couldn't flush stdout");
    
            let stdin = io::stdin();
            let line = stdin.lock().lines().next().expect("No line provided");
    
            match line {
                Ok(code) if !code.trim().is_empty() => self.run(code.trim_end().to_owned()),
                Err(_) => {
                    eprintln!("Error reading line from terminal");
                },
                _ => {
                    repl_active = false;
                }
            }
        }
    }
    
    fn run(&mut self, src: String) {
        let mut scanner = Scanner::new(src, self);
        let tokens = scanner.scan_tokens();
    
        for token in tokens.iter() {
            println!("{}", token);
        }

        if self.had_error {
            std::process::exit(65);
        }
    }
    
    pub fn error(&mut self, line: usize, msg: String) { 
        self.report(line, "".to_string(), msg);
    }
    
    pub fn report(&mut self, line: usize, location: String, msg: String) { 
        eprintln!("[line {}] Error{}: {}", line, location, msg);
        self.had_error = true;
    }
}