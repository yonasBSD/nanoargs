//! Example: Environment variable fallback for options
//!
//! Run with: cargo run --example env_fallback -- --output result.txt
//! Or set env vars and omit the flags:
//!   MYAPP_OUTPUT=from_env.txt MYAPP_FORMAT=json cargo run --example env_fallback

use nanoargs::{ArgBuilder, Opt, ParseError};

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

    let args: Vec<String> = std::env::args().skip(1).collect();
    match parser.parse(args) {
        Ok(result) => {
            println!("output:    {:?}", result.get_option("output"));
            println!("format:    {:?}", result.get_option("format"));
            println!("log-level: {:?}", result.get_option("log-level"));
        }
        Err(ParseError::HelpRequested(text)) => print!("{}", text),
        Err(e) => eprintln!("error: {}", e),
    }
}
