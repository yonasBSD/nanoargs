//! Example: Combined short flags and attached option values
//!
//! Run with:
//!   cargo run --example short_flags -- -abc
//!   cargo run --example short_flags -- -w10
//!   cargo run --example short_flags -- -w=10
//!   cargo run --example short_flags -- -abcw10
//!   cargo run --example short_flags -- -abcw 10
//!   cargo run --example short_flags -- -abcw=10

use nanoargs::{ArgBuilder, Flag, Opt, ParseError};

fn main() {
    let parser = ArgBuilder::new()
        .name("short-flags-demo")
        .description("Demonstrates combined short flags and attached option values")
        .flag(Flag::new("all").desc("Show all entries").short('a'))
        .flag(Flag::new("brief").desc("Use brief output format").short('b'))
        .flag(Flag::new("color").desc("Enable colored output").short('c'))
        .flag(Flag::new("verbose").desc("Enable verbose output").short('v'))
        .option(Opt::new("width").placeholder("NUM").desc("Set column width").short('w'))
        .build()
        .unwrap();

    let args: Vec<String> = std::env::args().skip(1).collect();
    match parser.parse(args) {
        Ok(result) => {
            println!("--- Flags ---");
            println!("  all:      {}", result.get_flag("all"));
            println!("  brief:    {}", result.get_flag("brief"));
            println!("  color:    {}", result.get_flag("color"));
            println!("  verbose:  {}", result.get_flag("verbose"));
            println!("\n--- Options ---");
            println!("  width:    {:?}", result.get_option("width"));
        }
        Err(ParseError::HelpRequested(text)) => print!("{}", text),
        Err(e) => eprintln!("error: {}", e),
    }
}
