mod lexer;
mod parser;
mod formatter;

use lexer::Lexer;
use parser::Parser;
use formatter::Formatter;
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: sfmt [--write] [--check] <file>");
        std::process::exit(1);
    }

    let mut write_mode = false;
    let mut check_mode = false;
    let mut file_path = None;

    for arg in &args[1..] {
        match arg.as_str() {
            "--write" => write_mode = true,
            "--check" => check_mode = true,
            _ => file_path = Some(arg.clone()),
        }
    }

    let file_path = match file_path {
        Some(p) => p,
        None => {
            eprintln!("Error: No file specified");
            std::process::exit(1);
        }
    };

    if !Path::new(&file_path).exists() {
        eprintln!("Error: File '{}' not found", file_path);
        std::process::exit(1);
    }

    let contents = match fs::read_to_string(&file_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            std::process::exit(1);
        }
    };

    // Lex and parse
    let mut lexer = Lexer::new(&contents);
    let tokens = lexer.tokenize();

    let mut parser = Parser::new(tokens);
    let nodes = match parser.parse() {
        Ok(n) => n,
        Err(e) => {
            eprintln!("Parse error: {}", e);
            std::process::exit(1);
        }
    };

    // Format
    let mut formatter = Formatter::new();
    let formatted = formatter.format(&nodes);

    if check_mode {
        if formatted == contents {
            println!("✓ {} is properly formatted", file_path);
        } else {
            eprintln!("✗ {} needs formatting", file_path);
            std::process::exit(1);
        }
    } else if write_mode {
        match fs::write(&file_path, &formatted) {
            Ok(_) => println!("Formatted {}", file_path),
            Err(e) => {
                eprintln!("Error writing file: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        println!("{}", formatted);
    }
}

