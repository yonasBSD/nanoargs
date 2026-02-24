//! Example: Using positional arguments
//!
//! Run with: cargo run --example positionals -- input.txt output.txt extra1 extra2

use nanoargs::{ArgBuilder, ParseError, Pos};

fn main() {
    let parser = ArgBuilder::new()
        .name("positionals-demo")
        .description("Demonstrates positional argument parsing")
        .positional(Pos::new("input").desc("Input file").required())
        .positional(Pos::new("output").desc("Output file (optional)"))
        .build()
        .unwrap();

    let args: Vec<String> = std::env::args().skip(1).collect();
    match parser.parse(args) {
        Ok(result) => {
            let pos = result.get_positionals();
            for (i, arg) in pos.iter().enumerate() {
                println!("positional[{}]: {}", i, arg);
            }
        }
        Err(ParseError::HelpRequested(text)) => print!("{}", text),
        Err(e) => eprintln!("error: {}", e),
    }
}
