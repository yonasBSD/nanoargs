//! Example: subcommands — git-style sub-operations
//!
//! Demonstrates creating subcommand parsers with global flags that are
//! parsed before the subcommand name. Uses `extract!` on subcommand
//! results for ergonomic typed extraction.
//!
//! Run with:
//!   cargo run --example subcommands -- build --release
//!   cargo run --example subcommands -- build --target x86_64
//!   cargo run --example subcommands -- test --verbose --filter integration
//!   cargo run --example subcommands -- --quiet build --release
//!   cargo run --example subcommands -- --help
//!   cargo run --example subcommands -- build --help

use nanoargs::{extract, ArgBuilder, Flag, Opt, ParseError};

fn main() {
    let build_parser = ArgBuilder::new()
        .name("build")
        .description("Compile the project")
        .flag(Flag::new("release").desc("Build in release mode").short('r'))
        .option(Opt::new("target").placeholder("TRIPLE").desc("Target triple").short('t').default("native"))
        .build()
        .unwrap();

    let test_parser = ArgBuilder::new()
        .name("test")
        .description("Run the test suite")
        .flag(Flag::new("verbose").desc("Show detailed test output").short('v'))
        .option(Opt::new("filter").placeholder("PATTERN").desc("Only run tests matching pattern").short('f'))
        .build()
        .unwrap();

    let parser = ArgBuilder::new()
        .name("myapp")
        .description("A demo CLI with subcommands")
        .flag(Flag::new("quiet").desc("Suppress non-essential output").short('q'))
        .subcommand("build", "Compile the project", build_parser)
        .subcommand("test", "Run the test suite", test_parser)
        .build()
        .unwrap();

    match parser.parse_env() {
        Ok(result) => {
            let quiet = result.get_flag("quiet");
            if quiet {
                println!("[quiet mode]");
            }

            match result.subcommand() {
                Some("build") => {
                    let sub = result.subcommand_result().unwrap();
                    let opts = extract!(sub, {
                        release: bool,
                        target: String = "native".into(),
                    })
                    .unwrap();
                    println!("Building...");
                    println!("  release: {}", opts.release);
                    println!("  target:  {}", opts.target);
                }
                Some("test") => {
                    let sub = result.subcommand_result().unwrap();
                    let opts = extract!(sub, {
                        verbose: bool,
                        filter: Option<String>,
                    })
                    .unwrap();
                    println!("Testing...");
                    println!("  verbose: {}", opts.verbose);
                    println!("  filter:  {:?}", opts.filter);
                }
                Some(other) => println!("unknown subcommand: {other}"),
                None => println!("no subcommand given"),
            }
        }
        Err(ParseError::HelpRequested(text)) => print!("{text}"),
        Err(e) => eprintln!("error: {e}"),
    }
}
