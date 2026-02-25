//! Example: Auto-generated help text
//!
//! Demonstrates both programmatic access to help text via `parser.help_text()`
//! and the `--help` flag trigger via `ParseError::HelpRequested`.
//!
//! Enable the optional `color` feature for ANSI-styled help output:
//!   cargo build --example help_text --features color
//!
//! Run with:
//!   cargo run --example help_text

use nanoargs::{ArgBuilder, Flag, Opt, ParseError, Pos};

fn main() {
    let parser = ArgBuilder::new()
        .name("myapp")
        .description("A sample CLI application")
        .flag(Flag::new("verbose").desc("Enable verbose output").short('v'))
        .flag(Flag::new("quiet").desc("Suppress non-error output").short('q'))
        .option(Opt::new("config").placeholder("PATH").desc("Config file path").short('c').required())
        .option(Opt::new("port").placeholder("NUM").desc("Server port").short('p').default("8080"))
        .option(Opt::new("format").placeholder("FMT").desc("Output format"))
        .positional(Pos::new("command").desc("Command to execute").required())
        .positional(Pos::new("args").desc("Arguments for the command").multi())
        .build()
        .unwrap();

    // 1. Programmatic access — call help_text() directly
    println!("=== Programmatic help text ===\n");
    println!("{}", parser.help_text());

    // 2. Flag-triggered — parse synthetic args containing --help
    println!("=== Flag-triggered help (--help) ===\n");
    match parser.parse(vec!["--help".into()]) {
        Ok(_result) => {}
        Err(ParseError::HelpRequested(text)) => print!("{text}"),
        Err(e) => eprintln!("error: {e}"),
    }
}
