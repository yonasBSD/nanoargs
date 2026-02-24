//! Example: Error handling for invalid input
//!
//! Run with:
//!   cargo run --example error_handling              (missing required)
//!   cargo run --example error_handling -- --unknown (unknown argument)
//!   cargo run --example error_handling -- --output  (missing value)

use nanoargs::{ArgBuilder, Opt, ParseError};

fn main() {
    let parser = ArgBuilder::new()
        .name("error-demo")
        .option(Opt::new("output").placeholder("FILE").desc("Output file").short('o').required())
        .build()
        .unwrap();

    let args: Vec<String> = std::env::args().skip(1).collect();
    match parser.parse(args) {
        Ok(result) => {
            println!("output: {:?}", result.get_option("output"));
        }
        Err(ParseError::HelpRequested(text)) => print!("{}", text),
        Err(ParseError::MissingRequired(name)) => {
            eprintln!("Missing required argument: {}", name);
        }
        Err(ParseError::MissingValue(name)) => {
            eprintln!("Option needs a value: --{}", name);
        }
        Err(ParseError::UnknownArgument(token)) => {
            eprintln!("Unknown argument: {}", token);
        }
        Err(e) => eprintln!("error: {}", e),
    }
}
