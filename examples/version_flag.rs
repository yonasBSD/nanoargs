//! Example: Built-in version flag
//!
//! Run with: cargo run --example version_flag -- --version
//! Or:       cargo run --example version_flag -- -V

use nanoargs::{ArgBuilder, Flag, ParseError};

fn main() {
    let parser = ArgBuilder::new()
        .name("version-demo")
        .description("Demonstrates built-in version flag")
        .version(env!("CARGO_PKG_VERSION"))
        .flag(Flag::new("verbose").desc("Enable verbose output").short('v'))
        .build()
        .unwrap();

    let args: Vec<String> = std::env::args().skip(1).collect();
    match parser.parse(args) {
        Ok(result) => {
            println!("verbose: {}", result.get_flag("verbose"));
        }
        Err(ParseError::VersionRequested(text)) => println!("{}", text),
        Err(ParseError::HelpRequested(text)) => print!("{}", text),
        Err(e) => eprintln!("error: {}", e),
    }
}
