//! Example: Environment variable fallback
//!
//! Demonstrates the `.env()` modifier on options, which enables CLI > env > default
//! precedence. If a value is not provided on the command line, nanoargs checks the
//! named environment variable before falling back to a default (if configured).
//!
//! Run with:
//!   cargo run --example env_fallback -- --output result.txt
//!   cargo run --example env_fallback -- --output result.txt --format json --log-level debug
//!   MYAPP_OUTPUT=from_env.txt MYAPP_FORMAT=json cargo run --example env_fallback
//!   cargo run --example env_fallback -- --help

use nanoargs::{extract, ArgBuilder, Opt, ParseError};

fn main() {
    let parser = ArgBuilder::new()
        .name("env-fallback-demo")
        .description("Demonstrates environment variable fallback for options")
        .option(
            Opt::new("output").placeholder("FILE").desc("Output file path").short('o').env("MYAPP_OUTPUT").required(),
        )
        .option(
            Opt::new("format").placeholder("FMT").desc("Output format").short('f').env("MYAPP_FORMAT").default("text"),
        )
        .option(Opt::new("log-level").placeholder("LEVEL").desc("Log level").short('l').env("MYAPP_LOG_LEVEL"))
        .build()
        .unwrap();

    match parser.parse_env() {
        Ok(result) => {
            let opts = extract!(result, {
                output: String,
                format: String = "text".into(),
                log_level: Option<String>,
            })
            .unwrap();

            println!("output:    {}", opts.output);
            println!("format:    {}", opts.format);
            println!("log-level: {:?}", opts.log_level);
        }
        Err(ParseError::HelpRequested(text)) => print!("{text}"),
        Err(e) => eprintln!("error: {e}"),
    }
}
