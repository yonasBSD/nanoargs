//! Example: Using flags (boolean switches)
//!
//! Run with: cargo run --example flags -- --verbose --dry-run
//! Or:       cargo run --example flags -- -v
//! Or:       cargo run --example flags -- --debug (hidden flag, not shown in --help)
//! Try:      cargo run --example flags -- --help

use nanoargs::{ArgBuilder, Flag, ParseError};

fn main() {
    let parser = ArgBuilder::new()
        .name("flags-demo")
        .description("Demonstrates flag parsing")
        .flag(Flag::new("verbose").desc("Enable verbose output").short('v'))
        .flag(Flag::new("dry-run").desc("Simulate without making changes"))
        // Hidden flags parse normally but don't appear in --help output
        .flag(Flag::new("debug").desc("Enable debug mode").short('d').hidden())
        .build()
        .unwrap();

    let args: Vec<String> = std::env::args().skip(1).collect();
    match parser.parse(args) {
        Ok(result) => {
            println!("verbose: {}", result.get_flag("verbose"));
            println!("dry-run: {}", result.get_flag("dry-run"));
            println!("debug:   {}", result.get_flag("debug"));
        }
        Err(ParseError::HelpRequested(text)) => print!("{}", text),
        Err(e) => eprintln!("error: {}", e),
    }
}
