//! Example: extract! macro — the recommended nanoargs API
//!
//! The `extract!` macro provides one-step typed extraction of flags, options,
//! and positionals from a parse result into a struct. This is the recommended
//! approach for most CLI applications.
//!
//! Run with:
//!   cargo run --example extract -- --output result.txt --jobs 8 --tag=foo --tag=bar input.txt
//!   cargo run --example extract -- -o result.txt -j 8 --verbose --dry-run input.txt extra1 extra2
//!   cargo run --example extract -- --help
//!   cargo run --example extract -- --version

use nanoargs::{extract, ArgBuilder, Flag, Opt, ParseError, Pos};

fn main() {
    let parser = ArgBuilder::new()
        .name("extract-demo")
        .description("Demonstrates the extract! macro — the recommended nanoargs API")
        .version("1.0.0")
        .flag(Flag::new("verbose").desc("Enable verbose output").short('v'))
        .flag(Flag::new("dry-run").desc("Simulate without side effects"))
        .option(Opt::new("output").placeholder("FILE").desc("Output file path").short('o').required())
        .option(Opt::new("jobs").placeholder("NUM").desc("Number of parallel jobs").short('j').default("4"))
        .option(Opt::new("format").placeholder("FMT").desc("Output format").short('f'))
        .option(Opt::new("tag").placeholder("TAG").desc("Tags to apply (repeatable)").short('t').multi())
        .positional(Pos::new("input").desc("Input file to process").required())
        .positional(Pos::new("extra").desc("Additional arguments").multi())
        .build()
        .unwrap();

    match parser.parse_env() {
        Ok(result) => {
            // One-step typed extraction — all field types in one shot:
            //   bool          → flag lookup
            //   String        → required option
            //   T = default   → option with default value
            //   Option<T>     → optional option
            //   Vec<T>        → multi-value option
            //   T as @pos     → required positional
            //   Vec<T> as @pos → remaining positionals
            let opts = extract!(result, {
                verbose: bool,
                dry_run: bool,
                output: String,
                jobs: u32 = 4,
                format: Option<String>,
                tag: Vec<String>,
                input: String as @pos,
                extra: Vec<String> as @pos,
            })
            .unwrap();

            println!("verbose:  {}", opts.verbose);
            println!("dry_run:  {}", opts.dry_run);
            println!("output:   {}", opts.output);
            println!("jobs:     {}", opts.jobs);
            println!("format:   {:?}", opts.format);
            println!("tags:     {:?}", opts.tag);
            println!("input:    {}", opts.input);
            println!("extra:    {:?}", opts.extra);
        }
        Err(ParseError::HelpRequested(text)) => print!("{text}"),
        Err(ParseError::VersionRequested(text)) => println!("{text}"),
        Err(e) => eprintln!("error: {e}"),
    }
}
