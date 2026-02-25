//! Full demo example — showcases nanoargs features in one CLI.
//!
//! Used to generate the README demo GIF via `demo.tape`.
//!
//! Run with:
//!   cargo run --example full_demo --features color -- --help
//!   cargo run --example full_demo --features color -- --file data.csv --json --port 8080
//!   cargo run --example full_demo --features color -- --json --csv --file x
//!   cargo run --example full_demo --features color -- --file x --port 0
//!   cargo run --example full_demo --features color -- --version

use nanoargs::{extract, one_of, range, ArgBuilder, Flag, Opt, ParseError};

fn main() {
    let parser = ArgBuilder::new()
        .name("nanoargs-demo")
        .description("A zero-dependency CLI parser for Rust")
        .version("0.5.1")
        // Input sources — at least one required
        .flag(Flag::new("stdin").desc("Read from standard input").short('s'))
        .option(Opt::new("file").placeholder("PATH").desc("Read from a file").short('f'))
        // Output formats — mutually exclusive
        .flag(Flag::new("json").desc("Output as JSON"))
        .flag(Flag::new("csv").desc("Output as CSV"))
        .flag(Flag::new("yaml").desc("Output as YAML"))
        // Other options
        .flag(Flag::new("verbose").desc("Enable verbose output").short('v'))
        .option(
            Opt::new("port")
                .placeholder("NUM")
                .desc("Server port")
                .short('p')
                .default("8080")
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
        // Groups and conflicts
        .group("input source", &["stdin", "file"])
        .conflict("output format", &["json", "csv", "yaml"])
        .build()
        .unwrap();

    match parser.parse_env() {
        Ok(result) => {
            let opts = extract!(result, {
                stdin: bool,
                file: Option<String>,
                json: bool,
                csv: bool,
                yaml: bool,
                verbose: bool,
                port: u16 = 8080,
                level: String = "info".into(),
            })
            .unwrap();

            let source = if opts.stdin {
                "stdin".into()
            } else {
                format!("{}", opts.file.unwrap())
            };
            let format = if opts.json {
                "JSON"
            } else if opts.csv {
                "CSV"
            } else if opts.yaml {
                "YAML"
            } else {
                "text"
            };

            if opts.verbose {
                println!("[verbose mode]");
            }
            println!("source: {source}");
            println!("format: {format}");
            println!("port:   {}", opts.port);
            println!("level:  {}", opts.level);
        }
        Err(ParseError::HelpRequested(text)) => print!("{text}"),
        Err(ParseError::VersionRequested(text)) => println!("{text}"),
        Err(e) => eprintln!("{e}"),
    }
}
