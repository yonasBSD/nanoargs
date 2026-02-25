//! Example: Using options (key-value arguments)
//!
//! Run with: cargo run --example options -- --output result.txt --jobs 8 --include src --include tests
//! Or:       cargo run --example options -- -o=result.txt -j 8 --include=lib

use nanoargs::{ArgBuilder, Opt, ParseError};

fn main() {
    let parser = ArgBuilder::new()
        .name("options-demo")
        .description("Demonstrates option parsing with defaults and required values")
        .option(Opt::new("output").placeholder("FILE").desc("Output file path").short('o').required())
        .option(Opt::new("jobs").placeholder("NUM").desc("Number of parallel jobs").short('j').default("4"))
        .option(Opt::new("format").placeholder("FMT").desc("Output format").short('f'))
        // Multi-value option (repeatable)
        .option(Opt::new("include").placeholder("DIR").desc("Directories to include (repeatable)").short('i').multi())
        // Hidden option — parses normally but excluded from --help
        .option(Opt::new("trace-id").placeholder("ID").desc("Internal trace identifier").hidden())
        .build()
        .unwrap();

    let args: Vec<String> = std::env::args().skip(1).collect();
    match parser.parse(args) {
        Ok(result) => {
            println!("output: {:?}", result.get_option("output"));
            println!("format: {:?}", result.get_option("format"));

            // Multi-value: collect all --include values
            let includes = result.get_option_values("include");
            println!("include dirs: {:?}", includes);

            // Hidden option still accessible
            println!("trace-id: {:?} (hidden)", result.get_option("trace-id"));

            // Typed parsing with default fallback
            let jobs: u32 = result.get_option_or_default("jobs", 4).unwrap();
            println!("jobs (u32): {}", jobs);
        }
        Err(ParseError::HelpRequested(text)) => print!("{}", text),
        Err(e) => eprintln!("error: {}", e),
    }
}
