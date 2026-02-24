//! Example: Using subcommands (git-style sub-operations)
//!
//! Run with: cargo run --example subcommands -- build --release
//!           cargo run --example subcommands -- test --verbose
//!           cargo run --example subcommands -- --help
//!           cargo run --example subcommands -- build --help

use nanoargs::{ArgBuilder, Flag, Opt, ParseError};

fn main() {
    // Build subcommand parser
    let build_parser = ArgBuilder::new()
        .name("build")
        .description("Compile the project")
        .flag(Flag::new("release").desc("Build in release mode").short('r'))
        .option(Opt::new("target").placeholder("TRIPLE").desc("Target triple").short('t').default("native"))
        .build()
        .unwrap();

    // Test subcommand parser
    let test_parser = ArgBuilder::new()
        .name("test")
        .description("Run the test suite")
        .flag(Flag::new("verbose").desc("Show detailed test output").short('v'))
        .option(Opt::new("filter").placeholder("PATTERN").desc("Only run tests matching pattern").short('f'))
        .build()
        .unwrap();

    // Parent parser with global flags and subcommands
    let parser = ArgBuilder::new()
        .name("myapp")
        .description("A demo CLI with subcommands")
        .flag(Flag::new("quiet").desc("Suppress non-essential output").short('q'))
        .subcommand("build", "Compile the project", build_parser)
        .subcommand("test", "Run the test suite", test_parser)
        .build()
        .unwrap();

    let args: Vec<String> = std::env::args().skip(1).collect();
    match parser.parse(args) {
        Ok(result) => {
            let quiet = result.get_flag("quiet");
            if quiet {
                println!("[quiet mode]");
            }

            match result.subcommand() {
                Some("build") => {
                    let sub = result.subcommand_result().unwrap();
                    println!("Building...");
                    println!("  release: {}", sub.get_flag("release"));
                    println!("  target:  {:?}", sub.get_option("target"));
                }
                Some("test") => {
                    let sub = result.subcommand_result().unwrap();
                    println!("Testing...");
                    println!("  verbose: {}", sub.get_flag("verbose"));
                    println!("  filter:  {:?}", sub.get_option("filter"));
                }
                Some(other) => println!("unknown subcommand: {}", other),
                None => println!("no subcommand given"),
            }
        }
        Err(ParseError::HelpRequested(text)) => print!("{}", text),
        Err(e) => eprintln!("error: {}", e),
    }
}
