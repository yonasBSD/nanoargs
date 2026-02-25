//! Example: Argument groups and conflicts
//!
//! Groups require at least one member to be provided ("pick at least one").
//! Conflicts declare mutual exclusivity ("pick at most one").
//! These constraints are enforced during parsing, so `extract!` works naturally.
//!
//! Run with:
//!   cargo run --example groups_and_conflicts -- --file data.csv --json
//!   cargo run --example groups_and_conflicts -- --stdin --yaml
//!   cargo run --example groups_and_conflicts -- --json --csv          (ConflictViolation)
//!   cargo run --example groups_and_conflicts --                       (GroupViolation — no input source)
//!   cargo run --example groups_and_conflicts -- --help

use nanoargs::{extract, ArgBuilder, Flag, Opt, ParseError};

fn main() {
    let parser = ArgBuilder::new()
        .name("export")
        .description("Export data in various formats")
        .version("1.0.0")
        // Input sources — at least one must be provided
        .flag(Flag::new("stdin").desc("Read from standard input"))
        .option(Opt::new("file").placeholder("PATH").desc("Read from a file").short('f'))
        // Output formats — only one allowed at a time
        .flag(Flag::new("json").desc("Output as JSON"))
        .flag(Flag::new("csv").desc("Output as CSV"))
        .flag(Flag::new("yaml").desc("Output as YAML"))
        // Declare the group and conflict
        .group("input source", &["stdin", "file"])
        .conflict("output format", &["json", "csv", "yaml"])
        .build()
        .unwrap();

    // Groups and conflicts are validated during parse_env(), so by the time
    // we reach extract!, we know the constraints are satisfied.
    match parser.parse_env() {
        Ok(result) => {
            let opts = extract!(result, {
                stdin: bool,
                file: Option<String>,
                json: bool,
                csv: bool,
                yaml: bool,
            })
            .unwrap();

            let source = if opts.stdin {
                "stdin".to_string()
            } else {
                format!("file: {}", opts.file.unwrap())
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

            println!("Exporting from {source} as {format}");
        }
        Err(ParseError::HelpRequested(text)) => print!("{text}"),
        Err(ParseError::VersionRequested(text)) => println!("{text}"),
        Err(e) => eprintln!("{e}"),
    }
}
