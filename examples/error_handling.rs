//! Example: Error handling — matching on ParseError variants
//!
//! Shows how to match on each `ParseError` variant to provide user-friendly
//! error messages. This example deliberately defines a minimal parser so you
//! can trigger every variant from the command line.
//!
//! Run with:
//!   cargo run --example error_handling -- --output out.txt input.txt   (success)
//!   cargo run --example error_handling                                 (MissingRequired)
//!   cargo run --example error_handling -- --output                     (MissingValue)
//!   cargo run --example error_handling -- --unknown                    (UnknownArgument)
//!   cargo run --example error_handling -- --output a --output b input.txt  (DuplicateOption)
//!   cargo run --example error_handling -- --help                       (HelpRequested)
//!   cargo run --example error_handling -- --version                    (VersionRequested)

use nanoargs::{ArgBuilder, Opt, ParseError, Pos};

fn main() {
    let parser = ArgBuilder::new()
        .name("error-demo")
        .description("Demonstrates ParseError variant handling")
        .version("0.1.0")
        .option(Opt::new("output").placeholder("FILE").desc("Output file").short('o').required())
        .positional(Pos::new("input").desc("Input file").required())
        .build()
        .unwrap();

    match parser.parse_env() {
        Ok(result) => {
            println!("output: {:?}", result.get_option("output"));
            println!("input:  {:?}", result.get_positionals());
        }
        Err(ParseError::HelpRequested(text)) => print!("{text}"),
        Err(ParseError::VersionRequested(text)) => println!("{text}"),
        Err(ParseError::MissingRequired(name)) => {
            eprintln!("Missing required argument: {name}");
            eprintln!("Hint: provide --{name} or see --help for usage.");
        }
        Err(ParseError::MissingValue(name)) => {
            eprintln!("Option --{name} requires a value.");
            eprintln!("Hint: use --{name} <value>.");
        }
        Err(ParseError::UnknownArgument(token)) => {
            eprintln!("Unknown argument: {token}");
            eprintln!("Hint: run with --help to see available options.");
        }
        Err(ParseError::DuplicateOption(name)) => {
            eprintln!("Option --{name} was provided more than once.");
            eprintln!("Hint: this option only accepts a single value.");
        }
        Err(e) => eprintln!("error: {e}"),
    }
}
