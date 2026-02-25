//! Example: shell completions — generate tab-completion scripts
//!
//! Demonstrates how to generate a shell completion script from an `ArgParser`
//! schema. Pass the desired shell as the first argument.
//!
//! Run with:
//!   cargo run --example completions -- bash
//!   cargo run --example completions -- zsh
//!   cargo run --example completions -- fish
//!   cargo run --example completions -- powershell

use nanoargs::{ArgBuilder, Flag, Opt, Pos, Shell};

fn main() {
    // Build a sample CLI schema with flags, options, and subcommands.
    let build_sub = ArgBuilder::new()
        .name("build")
        .description("Compile the project")
        .flag(Flag::new("release").desc("Build in release mode").short('r'))
        .option(Opt::new("target").placeholder("TRIPLE").desc("Target triple").short('t'))
        .build()
        .unwrap();

    let test_sub = ArgBuilder::new()
        .name("test")
        .description("Run the test suite")
        .flag(Flag::new("verbose").desc("Show detailed output").short('v'))
        .option(Opt::new("filter").placeholder("PATTERN").desc("Only run matching tests").short('f'))
        .build()
        .unwrap();

    let parser = ArgBuilder::new()
        .name("myapp")
        .description("A demo CLI with completions support")
        .version("0.1.0")
        .flag(Flag::new("quiet").desc("Suppress non-essential output").short('q'))
        .option(Opt::new("config").placeholder("FILE").desc("Path to config file").short('c'))
        .positional(Pos::new("input").desc("Input file"))
        .subcommand("build", "Compile the project", build_sub)
        .subcommand("test", "Run the test suite", test_sub)
        .build()
        .unwrap();

    // Read the shell name from the first CLI argument.
    let shell_name = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: completions <bash|zsh|fish|powershell>");
        std::process::exit(1);
    });

    let shell: Shell = shell_name.parse().unwrap_or_else(|e| {
        eprintln!("error: {e}");
        std::process::exit(1);
    });

    // Generate and print the completion script.
    print!("{}", parser.generate_completions(shell));
}
