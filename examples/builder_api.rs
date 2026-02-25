//! Example: builder API — manual argument access for power users
//!
//! This example shows the lower-level builder API without the `extract!` macro.
//! Use this approach when you need fine-grained control over how parsed values
//! are accessed, or when `extract!` doesn't fit your use case.
//!
//! Run with:
//!   cargo run --example builder_api -- --output result.txt --jobs 8 -v --include=src --include=lib input.txt
//!   cargo run --example builder_api -- -o result.txt --verbose input.txt extra1 extra2
//!   cargo run --example builder_api -- --help

use nanoargs::{ArgBuilder, Flag, Opt, ParseError, Pos};

/// Placeholder for an expensive default computation.
fn num_cpus() -> u32 {
    std::thread::available_parallelism().map(|n| n.get() as u32).unwrap_or(1)
}

fn main() {
    let parser = ArgBuilder::new()
        .name("builder-demo")
        .description("Demonstrates the manual builder API for power users")
        .flag(Flag::new("verbose").desc("Enable verbose output").short('v'))
        .flag(Flag::new("debug").desc("Internal debug mode").hidden())
        .option(Opt::new("output").placeholder("FILE").desc("Output file path").short('o').required())
        .option(Opt::new("jobs").placeholder("NUM").desc("Number of parallel jobs").short('j').default("4"))
        .option(Opt::new("include").placeholder("DIR").desc("Directories to include (repeatable)").short('I').multi())
        .positional(Pos::new("input").desc("Input file to process").required())
        .positional(Pos::new("extra").desc("Additional arguments").multi())
        .build()
        .unwrap();

    match parser.parse_env() {
        Ok(result) => {
            // get_flag() returns bool
            let verbose = result.get_flag("verbose");
            let debug = result.get_flag("debug");

            // get_option_required::<T>() parses or returns OptionError
            let output: String = result.get_option_required("output").unwrap();

            // get_option_or_default::<T>() parses with a fallback
            let jobs: u32 = result.get_option_or_default("jobs", 4).unwrap();

            // get_option() returns Option<&str>
            let raw_jobs = result.get_option("jobs");

            // get_option_parsed::<T>() returns Option<Result<T, Err>> — None if absent
            let parsed_jobs: Option<Result<u32, _>> = result.get_option_parsed("jobs");

            // get_option_or::<T>() parses with a closure fallback (lazy default)
            let jobs_or: u32 = result.get_option_or("jobs", || num_cpus()).unwrap();

            // get_option_values() returns &[String] for multi-value options
            let includes = result.get_option_values("include");

            // get_option_values_parsed::<T>() returns Vec<Result<T, Err>> for typed multi-values
            let include_strs: Vec<Result<String, _>> = result.get_option_values_parsed("include");

            // get_option_values_or_default::<T>() parses multi-values with a fallback vec
            let includes_or: Vec<String> = result.get_option_values_or_default("include", vec!["src".into()]).unwrap();

            // get_positionals() returns &[String]
            let positionals = result.get_positionals();

            println!("verbose:      {verbose}");
            println!("debug:        {debug}");
            println!("output:       {output}");
            println!("jobs:         {jobs}");
            println!("raw jobs:     {raw_jobs:?}");
            println!("parsed jobs:  {parsed_jobs:?}");
            println!("jobs (or):    {jobs_or}");
            println!("includes:     {includes:?}");
            println!("includes (p): {include_strs:?}");
            println!("includes (d): {includes_or:?}");
            println!("positionals:  {positionals:?}");
        }
        Err(ParseError::HelpRequested(text)) => print!("{text}"),
        Err(e) => eprintln!("error: {e}"),
    }
}
