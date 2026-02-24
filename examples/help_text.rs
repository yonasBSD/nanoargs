//! Example: Auto-generated help text
//!
//! Run with: cargo run --example help_text -- --help
//!
//! For colored output, enable the `color` feature:
//!   cargo run --example help_text --features color -- --help

use nanoargs::{ArgBuilder, Flag, Opt, ParseError, Pos};

fn main() {
    let parser = ArgBuilder::new()
        .name("myapp")
        .description("A sample CLI application")
        .flag(Flag::new("verbose").desc("Enable verbose output").short('v'))
        .option(Opt::new("config").placeholder("PATH").desc("Config file path").short('c').required())
        .option(Opt::new("port").placeholder("NUM").desc("Server port").short('p').default("8080"))
        .positional(Pos::new("command").desc("Command to execute").required())
        .build()
        .unwrap();

    // Directly print the help text
    println!("=== Generated help text ===\n");
    println!("{}", parser.help_text());

    // Or trigger it via --help flag
    println!("=== Parsing with --help ===\n");
    match parser.parse(vec!["--help".into()]) {
        Err(ParseError::HelpRequested(text)) => print!("{}", text),
        _ => {}
    }
}
