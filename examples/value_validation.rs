//! Example: Value validation — constraining option and positional values
//!
//! Shows how to attach validators to options and positionals so that invalid
//! values are rejected during parsing with clear error messages. Includes
//! built-in `range()`, `one_of()`, `min_length()`, `max_length()`, and
//! `path_exists()` validators, plus a custom closure.
//!
//! Run with:
//!   cargo run --example value_validation -- --port 8080 --level info --tag hello /tmp
//!   cargo run --example value_validation -- --port 0 --level info --tag hello /tmp       (ValidationFailed: port out of range)
//!   cargo run --example value_validation -- --port 8080 --level trace --tag hello /tmp   (ValidationFailed: invalid level)
//!   cargo run --example value_validation -- --port 8080 --level info --tag ab /tmp       (ValidationFailed: tag too short)
//!   cargo run --example value_validation -- --port 8080 --level info --tag hello /no/such (ValidationFailed: path does not exist)
//!   cargo run --example value_validation -- --help

use nanoargs::{extract, max_length, min_length, one_of, path_exists, range, ArgBuilder, Opt, ParseError, Pos};

fn main() {
    let parser = ArgBuilder::new()
        .name("validation-demo")
        .description("Demonstrates value validation on options and positionals")
        .version("0.1.0")
        .option(
            Opt::new("port")
                .placeholder("NUM")
                .desc("Port number")
                .short('p')
                .default("3000")
                .validate(range(1, 65535)),
        )
        .option(
            Opt::new("level")
                .placeholder("LEVEL")
                .desc("Log level")
                .short('l')
                .default("info")
                .validate(one_of(&["debug", "info", "warn", "error"])),
        )
        .option(
            Opt::new("tag")
                .placeholder("TAG")
                .desc("Resource tag (3-20 chars)")
                .short('t')
                .validate(min_length(3))
                .validate(max_length(20)),
        )
        .positional(Pos::new("output").desc("Output directory (must exist)").required().validate(path_exists()))
        .build()
        .unwrap();

    match parser.parse_env() {
        Ok(result) => {
            let opts = extract!(result, {
                port: u16 = 3000,
                level: String = "info".into(),
                tag: Option<String>,
                output: String as @pos,
            })
            .unwrap();

            println!("port:   {}", opts.port);
            println!("level:  {}", opts.level);
            if let Some(tag) = &opts.tag {
                println!("tag:    {}", tag);
            }
            println!("output: {}", opts.output);
        }
        Err(ParseError::HelpRequested(text)) => print!("{text}"),
        Err(ParseError::VersionRequested(text)) => println!("{text}"),
        Err(e) => eprintln!("{e}"),
    }
}
