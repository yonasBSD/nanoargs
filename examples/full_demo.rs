//! Full demo: showcases all nanoargs features together
//!
//! Run with:
//!   cargo run --example full_demo -- --verbose -o=result.txt --jobs 8 --tag=foo --tag=bar input.txt extra1 extra2
//!   cargo run --example full_demo -- -vj8 -o=result.txt --tag=release input.txt
//!   cargo run --example full_demo -- --help
//!   cargo run --example full_demo -- -- --this-is-positional -also-positional

use nanoargs::{ArgBuilder, Flag, Opt, ParseError, Pos};

fn main() {
    let parser = ArgBuilder::new()
        .name("full-demo")
        .description("Demonstrates all nanoargs features in one example")
        // Flags
        .flag(Flag::new("verbose").desc("Enable verbose output").short('v'))
        .flag(Flag::new("dry-run").desc("Simulate without side effects"))
        // Hidden flag — parses normally but excluded from --help
        .flag(Flag::new("debug").desc("Enable debug internals").short('d').hidden())
        // Options (required, default, optional)
        .option(Opt::new("output").placeholder("FILE").desc("Output file path").short('o').required())
        .option(Opt::new("jobs").placeholder("NUM").desc("Number of parallel jobs").short('j').default("4"))
        .option(Opt::new("format").placeholder("FMT").desc("Output format (json, csv, text)").short('f'))
        // Hidden option — parses normally but excluded from --help
        .option(Opt::new("trace-id").placeholder("ID").desc("Internal trace identifier").hidden())
        // Multi-value option
        .option(Opt::new("tag").placeholder("TAG").desc("Tags to apply (repeatable)").short('t').multi())
        // Positionals
        .positional(Pos::new("input").desc("Input file to process").required())
        .positional(Pos::new("extra").desc("Additional arguments"))
        .build()
        .unwrap();

    let args: Vec<String> = std::env::args().skip(1).collect();

    match parser.parse(args) {
        Ok(result) => {
            println!("--- Flags ---");
            println!("  verbose:  {}", result.get_flag("verbose"));
            println!("  dry-run:  {}", result.get_flag("dry-run"));
            println!("  debug:    {} (hidden)", result.get_flag("debug"));

            println!("\n--- Options ---");
            println!("  output:   {:?}", result.get_option("output"));
            println!("  format:   {:?}", result.get_option("format"));
            println!("  trace-id: {:?} (hidden)", result.get_option("trace-id"));
            println!("  tags:     {:?}", result.get_option_values("tag"));

            // Typed parsing with default fallback
            let jobs: u32 = result.get_option_or_default("jobs", 4).unwrap();
            println!("  jobs:     {} (parsed as u32)", jobs);

            println!("\n--- Positionals ---");
            for (i, p) in result.get_positionals().iter().enumerate() {
                println!("  [{}]: {}", i, p);
            }
        }
        Err(ParseError::HelpRequested(text)) => {
            print!("{}", text);
        }
        Err(e) => {
            eprintln!("error: {}", e);
            eprintln!("\nRun with --help for usage information.");
            std::process::exit(1);
        }
    }
}
