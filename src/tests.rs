use super::*;

/// Strip ANSI escape sequences so assertions work under both plain and colored builds.
fn strip_ansi(s: &str) -> String {
    let mut out = String::new();
    let mut in_escape = false;
    for ch in s.chars() {
        if in_escape {
            if ch.is_ascii_alphabetic() {
                in_escape = false;
            }
        } else if ch == '\x1b' {
            in_escape = true;
        } else {
            out.push(ch);
        }
    }
    out
}

fn parser_with_flags_and_option() -> ArgParser {
    ArgBuilder::new()
        .flag(Flag::new("alpha").desc("alpha flag").short('a'))
        .flag(Flag::new("bravo").desc("bravo flag").short('b'))
        .flag(Flag::new("charlie").desc("charlie flag").short('c'))
        .option(Opt::new("width").placeholder("N").desc("width option").short('w'))
        .build()
        .unwrap()
}

#[test]
fn combined_short_flags_abc() {
    let parser = parser_with_flags_and_option();
    let result = parser.parse(vec!["-abc".into()]).unwrap();
    assert!(result.get_flag("alpha"));
    assert!(result.get_flag("bravo"));
    assert!(result.get_flag("charlie"));
}

#[test]
fn attached_value_w10() {
    let parser = parser_with_flags_and_option();
    let result = parser.parse(vec!["-w10".into()]).unwrap();
    assert_eq!(result.get_option("width"), Some("10"));
}

#[test]
fn combined_flags_then_option_with_attached_value() {
    let parser = parser_with_flags_and_option();
    let result = parser.parse(vec!["-abcw10".into()]).unwrap();
    assert!(result.get_flag("alpha"));
    assert!(result.get_flag("bravo"));
    assert!(result.get_flag("charlie"));
    assert_eq!(result.get_option("width"), Some("10"));
}

#[test]
fn combined_flags_then_option_next_token() {
    let parser = parser_with_flags_and_option();
    let result = parser.parse(vec!["-abcw".into(), "10".into()]).unwrap();
    assert!(result.get_flag("alpha"));
    assert!(result.get_flag("bravo"));
    assert!(result.get_flag("charlie"));
    assert_eq!(result.get_option("width"), Some("10"));
}

#[test]
fn repeated_flag_vvv() {
    let parser = ArgBuilder::new().flag(Flag::new("verbose").desc("verbose flag").short('v')).build().unwrap();
    let result = parser.parse(vec!["-vvv".into()]).unwrap();
    assert!(result.get_flag("verbose"));
}

#[test]
fn unknown_char_in_cluster() {
    let parser = parser_with_flags_and_option();
    let err = parser.parse(vec!["-abx".into()]).unwrap_err();
    match err {
        ParseError::UnknownArgument(msg) => {
            assert!(msg.contains('x'), "error should mention unknown char 'x': {msg}");
        }
        other => panic!("expected UnknownArgument, got: {other}"),
    }
}

#[test]
fn missing_value_option_at_end_of_cluster() {
    let parser = parser_with_flags_and_option();
    let err = parser.parse(vec!["-abcw".into()]).unwrap_err();
    match err {
        ParseError::MissingValue(name) => assert_eq!(name, "width"),
        other => panic!("expected MissingValue, got: {other}"),
    }
}

#[test]
fn combined_token_after_double_dash_is_positional() {
    let parser = parser_with_flags_and_option();
    let result = parser.parse(vec!["--".into(), "-abc".into()]).unwrap();
    assert!(!result.get_flag("alpha"));
    assert!(!result.get_flag("bravo"));
    assert!(!result.get_flag("charlie"));
    assert_eq!(result.get_positionals(), &["-abc".to_string()]);
}

// --- Subcommand parsing edge case tests ---

fn parent_with_subcommands() -> ArgParser {
    let build_parser =
        ArgBuilder::new().flag(Flag::new("release").desc("Build in release mode").short('r')).build().unwrap();
    let test_parser =
        ArgBuilder::new().flag(Flag::new("verbose").desc("Verbose test output").short('v')).build().unwrap();
    ArgBuilder::new()
        .name("myapp")
        .flag(Flag::new("debug").desc("Enable debug mode").short('d'))
        .subcommand("build", "Compile the project", build_parser)
        .subcommand("test", "Run tests", test_parser)
        .build()
        .unwrap()
}

#[test]
fn subcommand_empty_args_returns_no_subcommand() {
    let parser = parent_with_subcommands();
    let err = parser.parse(vec![]).unwrap_err();
    match err {
        ParseError::NoSubcommand(names) => {
            assert!(names.contains("build"));
            assert!(names.contains("test"));
        }
        other => panic!("expected NoSubcommand, got: {other}"),
    }
}

#[test]
fn subcommand_unknown_name_returns_unknown_subcommand() {
    let parser = parent_with_subcommands();
    let err = parser.parse(vec!["deploy".into()]).unwrap_err();
    assert_eq!(err, ParseError::UnknownSubcommand("deploy".into()));
}

#[test]
fn subcommand_unknown_flag_after_subcommand() {
    let parser = parent_with_subcommands();
    let err = parser.parse(vec!["build".into(), "--unknown".into()]).unwrap_err();
    match err {
        ParseError::UnknownArgument(token) => assert_eq!(token, "--unknown"),
        other => panic!("expected UnknownArgument, got: {other}"),
    }
}

#[test]
fn subcommand_help_before_subcommand_returns_parent_help() {
    let parser = parent_with_subcommands();
    let err = parser.parse(vec!["--help".into()]).unwrap_err();
    match err {
        ParseError::HelpRequested(text) => {
            assert!(text.contains("myapp"), "parent help should contain program name");
        }
        other => panic!("expected HelpRequested, got: {other}"),
    }
}

#[test]
fn subcommand_help_after_subcommand_returns_subcommand_help() {
    let parser = parent_with_subcommands();
    let err = parser.parse(vec!["build".into(), "--help".into()]).unwrap_err();
    match err {
        ParseError::HelpRequested(text) => {
            // Subcommand help should mention the subcommand's own flags
            assert!(text.contains("release"), "subcommand help should contain its flags");
        }
        other => panic!("expected HelpRequested, got: {other}"),
    }
}

// --- Duplicate argument validation tests ---

#[test]
fn duplicate_short_flag_flag() {
    let err = ArgBuilder::new()
        .flag(Flag::new("verbose").desc("verbose").short('v'))
        .flag(Flag::new("version").desc("version").short('v'))
        .build()
        .unwrap_err();
    assert!(
        matches!(err, ParseError::InvalidFormat(ref msg) if msg.contains("-v")),
        "expected duplicate short error, got: {err}"
    );
}

#[test]
fn duplicate_short_flag_option() {
    let err = ArgBuilder::new()
        .flag(Flag::new("verbose").desc("verbose").short('v'))
        .option(Opt::new("value").placeholder("VAL").desc("a value").short('v'))
        .build()
        .unwrap_err();
    assert!(
        matches!(err, ParseError::InvalidFormat(ref msg) if msg.contains("-v")),
        "expected duplicate short error, got: {err}"
    );
}

#[test]
fn duplicate_long_flag_option() {
    let err = ArgBuilder::new()
        .flag(Flag::new("output").desc("output flag"))
        .option(Opt::new("output").placeholder("FILE").desc("output option"))
        .build()
        .unwrap_err();
    assert!(
        matches!(err, ParseError::InvalidFormat(ref msg) if msg.contains("--output")),
        "expected duplicate long error, got: {err}"
    );
}

#[test]
fn no_duplicate_when_shorts_differ() {
    ArgBuilder::new()
        .flag(Flag::new("verbose").desc("verbose").short('v'))
        .flag(Flag::new("debug").desc("debug").short('d'))
        .option(Opt::new("output").placeholder("FILE").desc("output").short('o'))
        .build()
        .unwrap();
}

// --- Help text alignment tests ---

#[test]
fn help_single_flag_has_column_gap() {
    let parser =
        ArgBuilder::new().name("app").flag(Flag::new("verbose").desc("Enable verbose").short('v')).build().unwrap();
    let help = strip_ansi(&parser.help_text());
    assert!(help.contains("  -v, --verbose  Enable verbose\n"));
}

#[test]
fn help_flags_mixed_short_aligned() {
    let parser = ArgBuilder::new()
        .name("app")
        .flag(Flag::new("v").desc("short name").short('v'))
        .flag(Flag::new("very-long-flag").desc("long name"))
        .build()
        .unwrap();
    let help = strip_ansi(&parser.help_text());
    let lines: Vec<&str> = help.lines().filter(|l| l.starts_with("  ")).filter(|l| l.contains("--")).collect();
    assert_eq!(lines.len(), 2);
    // Both descriptions should start at the same column
    let col0 = lines[0].find("short name").unwrap();
    let col1 = lines[1].find("long name").unwrap();
    assert_eq!(col0, col1, "descriptions should be aligned: {:?}", lines);
}

#[test]
fn help_empty_parser_no_sections() {
    let parser = ArgBuilder::new().name("app").description("My app").build().unwrap();
    let help = strip_ansi(&parser.help_text());
    assert!(!help.contains("Options:"));
    assert!(!help.contains("Positional arguments:"));
    assert!(!help.contains("Subcommands:"));
    assert!(help.contains("My app"));
    assert!(help.contains("Usage: app"));
}

#[test]
fn help_varying_option_lengths_aligned() {
    let parser = ArgBuilder::new()
        .name("app")
        .option(Opt::new("o").placeholder("FILE").desc("output file").short('o'))
        .option(Opt::new("very-long-option").placeholder("VAL").desc("some value"))
        .build()
        .unwrap();
    let help = strip_ansi(&parser.help_text());
    let option_lines: Vec<&str> =
        help.lines().skip_while(|l| !l.contains("Options:")).skip(1).take_while(|l| l.starts_with("  ")).collect();
    assert_eq!(option_lines.len(), 2);
    let col0 = option_lines[0].find("output file").unwrap();
    let col1 = option_lines[1].find("some value").unwrap();
    assert_eq!(col0, col1, "option descriptions should be aligned: {:?}", option_lines);
}

// --- Duplicate option tests ---

#[test]
fn duplicate_non_multi_option_errors() {
    let parser = ArgBuilder::new()
        .option(Opt::new("output").placeholder("FILE").desc("Output file").short('o'))
        .build()
        .unwrap();
    let err = parser
        .parse(vec![
            "--output".into(),
            "a.txt".into(),
            "--output".into(),
            "b.txt".into(),
        ])
        .unwrap_err();
    assert!(
        matches!(err, ParseError::DuplicateOption(ref name) if name == "output"),
        "expected DuplicateOption, got: {err}"
    );
}

#[test]
fn duplicate_multi_option_is_allowed() {
    let parser =
        ArgBuilder::new().option(Opt::new("tag").placeholder("TAG").desc("A tag").short('t').multi()).build().unwrap();
    let result = parser.parse(vec!["--tag".into(), "a".into(), "--tag".into(), "b".into()]).unwrap();
    assert_eq!(result.get_option_values("tag"), &["a", "b"]);
}

// --- Version flag interaction tests ---

#[test]
fn help_before_version_returns_help() {
    let parser = ArgBuilder::new().name("myapp").version("1.0.0").build().unwrap();
    let err = parser.parse(vec!["--help".into(), "--version".into()]).unwrap_err();
    assert!(
        matches!(err, ParseError::HelpRequested(_)),
        "expected HelpRequested, got: {err}"
    );
}

#[test]
fn version_before_help_returns_version() {
    let parser = ArgBuilder::new().name("myapp").version("1.0.0").build().unwrap();
    let err = parser.parse(vec!["--version".into(), "--help".into()]).unwrap_err();
    match err {
        ParseError::VersionRequested(text) => assert_eq!(text, "myapp 1.0.0"),
        other => panic!("expected VersionRequested, got: {other}"),
    }
}

#[test]
fn double_dash_then_version_is_positional() {
    let parser = ArgBuilder::new().version("1.0.0").build().unwrap();
    let result = parser.parse(vec!["--".into(), "--version".into()]).unwrap();
    assert_eq!(result.get_positionals(), &["--version".to_string()]);
}

#[test]
fn double_dash_then_short_v_is_positional() {
    let parser = ArgBuilder::new().version("1.0.0").build().unwrap();
    let result = parser.parse(vec!["--".into(), "-V".into()]).unwrap();
    assert_eq!(result.get_positionals(), &["-V".to_string()]);
}

#[test]
fn version_before_subcommand_returns_parent_version() {
    let sub = ArgBuilder::new().flag(Flag::new("verbose").desc("verbose").short('v')).build().unwrap();
    let parser = ArgBuilder::new().name("myapp").version("2.0.0").subcommand("run", "Run stuff", sub).build().unwrap();
    let err = parser.parse(vec!["--version".into(), "run".into()]).unwrap_err();
    match err {
        ParseError::VersionRequested(text) => assert_eq!(text, "myapp 2.0.0"),
        other => panic!("expected VersionRequested, got: {other}"),
    }
}

#[test]
fn version_after_subcommand_handled_by_subcommand() {
    let sub = ArgBuilder::new().version("0.1.0").build().unwrap();
    let parser = ArgBuilder::new().name("myapp").version("2.0.0").subcommand("run", "Run stuff", sub).build().unwrap();
    let err = parser.parse(vec!["run".into(), "--version".into()]).unwrap_err();
    match err {
        ParseError::VersionRequested(text) => {
            // Subcommand has no name, so just version string
            assert_eq!(text, "0.1.0");
        }
        other => panic!("expected VersionRequested from subcommand, got: {other}"),
    }
}

// ── free::parse_loose_from coverage ──────────────────────────────────────

#[test]
fn loose_long_option_with_value() {
    let args = vec!["--output".into(), "file.txt".into()];
    let result = crate::free::parse_loose_from(args).unwrap();
    assert_eq!(result.get_option("output"), Some("file.txt"));
}

#[test]
fn loose_long_flag_no_value() {
    let args = vec!["--verbose".into(), "--debug".into()];
    let result = crate::free::parse_loose_from(args).unwrap();
    assert!(result.get_flag("verbose"));
    assert!(result.get_flag("debug"));
}

#[test]
fn loose_long_option_equals_syntax() {
    let args = vec!["--output=file.txt".into()];
    let result = crate::free::parse_loose_from(args).unwrap();
    assert_eq!(result.get_option("output"), Some("file.txt"));
}

#[test]
fn loose_short_option_with_value() {
    let args = vec!["-o".into(), "file.txt".into()];
    let result = crate::free::parse_loose_from(args).unwrap();
    assert_eq!(result.get_option("o"), Some("file.txt"));
}

#[test]
fn loose_short_flag_no_value() {
    let args = vec!["-v".into(), "-d".into()];
    let result = crate::free::parse_loose_from(args).unwrap();
    assert!(result.get_flag("v"));
    assert!(result.get_flag("d"));
}

#[test]
fn loose_short_option_equals_syntax() {
    let args = vec!["-o=file.txt".into()];
    let result = crate::free::parse_loose_from(args).unwrap();
    assert_eq!(result.get_option("o"), Some("file.txt"));
}

#[test]
fn loose_positional_args() {
    let args = vec!["hello".into(), "world".into()];
    let result = crate::free::parse_loose_from(args).unwrap();
    assert_eq!(result.get_positionals(), &["hello", "world"]);
}

#[test]
fn loose_double_dash_stops_parsing() {
    let args = vec!["--verbose".into(), "--".into(), "--not-a-flag".into()];
    let result = crate::free::parse_loose_from(args).unwrap();
    assert!(result.get_flag("verbose"));
    assert_eq!(result.get_positionals(), &["--not-a-flag"]);
}

#[test]
fn loose_help_flag_returns_error() {
    let args = vec!["--help".into()];
    let err = crate::free::parse_loose_from(args).unwrap_err();
    match err {
        ParseError::HelpRequested(text) => assert!(text.contains("Usage")),
        other => panic!("expected HelpRequested, got: {other}"),
    }
}

#[test]
fn loose_short_help_flag_returns_error() {
    let args = vec!["-h".into()];
    let err = crate::free::parse_loose_from(args).unwrap_err();
    match err {
        ParseError::HelpRequested(_) => {}
        other => panic!("expected HelpRequested, got: {other}"),
    }
}

#[test]
fn loose_mixed_args() {
    let args = vec![
        "--verbose".into(),
        "-o".into(),
        "out.txt".into(),
        "positional".into(),
        "--".into(),
        "--rest".into(),
    ];
    let result = crate::free::parse_loose_from(args).unwrap();
    assert!(result.get_flag("verbose"));
    assert_eq!(result.get_option("o"), Some("out.txt"));
    assert_eq!(result.get_positionals(), &["positional", "--rest"]);
}

#[test]
fn loose_empty_args() {
    let result = crate::free::parse_loose_from(vec![]).unwrap();
    assert!(result.get_positionals().is_empty());
    assert!(!result.get_flag("anything"));
}

#[test]
fn loose_flag_when_next_starts_with_dash() {
    // --output followed by -v should treat --output as a flag
    let args = vec!["--output".into(), "-v".into()];
    let result = crate::free::parse_loose_from(args).unwrap();
    assert!(result.get_flag("output"));
    assert!(result.get_flag("v"));
}

// ── ParseResultBuilder coverage ──────────────────────────────────────────

#[test]
fn result_builder_new_and_multi_option() {
    let result =
        ParseResultBuilder::new().multi_option("tags", "a").multi_option("tags", "b").multi_option("tags", "c").build();
    assert_eq!(result.get_option_values("tags"), &["a", "b", "c"]);
    // get_option returns last value
    assert_eq!(result.get_option("tags"), Some("c"));
}

#[test]
fn result_builder_subcommand() {
    let sub = ParseResultBuilder::new().flag("verbose", true).build();
    let result = ParseResultBuilder::new().subcommand("run", sub).build();
    assert_eq!(result.subcommand(), Some("run"));
    assert!(result.subcommand_result().unwrap().get_flag("verbose"));
}

// ── OptionError Display coverage ─────────────────────────────────────────

#[test]
fn option_error_missing_display() {
    let err = OptionError::Missing { option: "port".into() };
    assert_eq!(err.to_string(), "option --port is required but was not provided");
}

#[test]
fn option_error_parse_failed_display() {
    let err = OptionError::ParseFailed {
        option: "port".into(),
        message: "invalid digit".into(),
    };
    assert_eq!(err.to_string(), "option --port: invalid digit");
}

#[test]
fn option_error_is_std_error() {
    let err = OptionError::Missing { option: "x".into() };
    // Verify it implements std::error::Error
    let _: &dyn std::error::Error = &err;
}
