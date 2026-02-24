#![cfg(feature = "color")]

mod common;

use common::*;
use nanoargs::{ArgBuilder, Flag, Opt, Pos};
use proptest::prelude::*;
use std::collections::HashSet;

/// Build a parser from generated parts, deduplicating names and shorts to avoid build errors.
fn build_parser(
    flags: &[(String, Option<char>, String)],
    options: &[(String, Option<char>, String, String, bool, Option<String>)],
    positionals: &[(String, String, bool)],
) -> Option<nanoargs::ArgParser> {
    let mut used_longs = HashSet::new();
    let mut used_shorts = HashSet::new();
    used_shorts.insert('h'); // reserved for help

    let mut builder = ArgBuilder::new().name("testapp").description("Test app");

    for (long, short, desc) in flags {
        if !used_longs.insert(long.clone()) {
            continue;
        }
        let s = short.filter(|c| used_shorts.insert(*c));
        let mut f = Flag::new(long).desc(desc);
        if let Some(ch) = s {
            f = f.short(ch);
        }
        builder = builder.flag(f);
    }
    for (long, short, placeholder, desc, required, default) in options {
        if !used_longs.insert(long.clone()) {
            continue;
        }
        let s = short.filter(|c| used_shorts.insert(*c));
        let mut o = Opt::new(long).placeholder(placeholder).desc(desc);
        if let Some(ch) = s {
            o = o.short(ch);
        }
        if *required {
            o = o.required();
        }
        if let Some(d) = default {
            o = o.default(d);
        }
        builder = builder.option(o);
    }
    for (name, desc, required) in positionals {
        let mut p = Pos::new(name).desc(desc);
        if *required {
            p = p.required();
        }
        builder = builder.positional(p);
    }

    builder.build().ok()
}

/// Build a parser and return colored help text.
fn colored_help_from(
    flags: &[(String, Option<char>, String)],
    options: &[(String, Option<char>, String, String, bool, Option<String>)],
    positionals: &[(String, String, bool)],
) -> Option<String> {
    build_parser(flags, options, positionals).map(|p| nanocolor::with_colors_override(true, || p.help_text()))
}

/// Build a parser and return plain help text.
fn plain_help_from(
    flags: &[(String, Option<char>, String)],
    options: &[(String, Option<char>, String, String, bool, Option<String>)],
    positionals: &[(String, String, bool)],
) -> Option<String> {
    build_parser(flags, options, positionals).map(|p| nanocolor::with_colors_override(false, || p.help_text()))
}

// Alignment preservation under colorization
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop1_alignment_preservation(
        flags in prop::collection::vec(arb_safe_flag_def(), 0..4),
        options in prop::collection::vec(arb_safe_option_def(), 0..4),
        positionals in prop::collection::vec(arb_safe_positional_def(), 0..3),
    ) {
        let colored = colored_help_from(&flags, &options, &positionals);
        let plain = plain_help_from(&flags, &options, &positionals);

        if let (Some(colored), Some(plain)) = (colored, plain) {
            let stripped = strip_ansi(&colored);
            for (i, (s_line, p_line)) in stripped.lines().zip(plain.lines()).enumerate() {
                let s_indent = s_line.len() - s_line.trim_start().len();
                let p_indent = p_line.len() - p_line.trim_start().len();
                prop_assert_eq!(s_indent, p_indent,
                    "Line {} indent mismatch: stripped={:?} plain={:?}", i, s_line, p_line);
            }
        }
    }
}

// Flag/option names are green and placeholders are cyan
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop2_flag_option_names_green_placeholders_cyan(
        flags in prop::collection::vec(arb_safe_flag_def(), 1..4),
        options in prop::collection::vec(arb_safe_option_def(), 1..3),
    ) {
        if let Some(help) = colored_help_from(&flags, &options, &[]) {
            let mut used_longs = HashSet::new();
            used_longs.insert("help".to_string());
            let mut flag_longs = Vec::new();
            for (long, _, _) in &flags {
                if used_longs.insert(long.clone()) {
                    flag_longs.push(long.clone());
                }
            }
            let mut opt_longs = Vec::new();
            let mut opt_placeholders = Vec::new();
            for (long, _, placeholder, _, _, _) in &options {
                if used_longs.insert(long.clone()) {
                    opt_longs.push(long.clone());
                    opt_placeholders.push(placeholder.clone());
                }
            }

            for long in &flag_longs {
                prop_assert!(contains_green(&help, &format!("--{long}")),
                    "Flag --{} should be green", long);
            }
            for (long, ph) in opt_longs.iter().zip(opt_placeholders.iter()) {
                prop_assert!(contains_green(&help, &format!("--{long}")),
                    "Option --{} should be green", long);
                prop_assert!(contains_cyan(&help, &format!("<{ph}>")),
                    "Placeholder <{}> should be cyan", ph);
            }
        }
    }
}

// Metadata annotations are dim
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop3_metadata_annotations_dim(
        long in arb_safe_identifier(),
        short in arb_short(),
        placeholder in arb_safe_identifier(),
        desc in arb_safe_description(),
        env_var in arb_env_var_name(),
        default_val in arb_safe_identifier(),
    ) {
        let s = short.filter(|&c| c != 'h');
        let mut o = Opt::new(&long).placeholder(&placeholder).desc(&desc).required().default(&default_val).env(&env_var);
        if let Some(ch) = s { o = o.short(ch); }
        let parser = ArgBuilder::new()
            .name("testapp")
            .option(o)
            .build()
            .unwrap();

        let help = nanocolor::with_colors_override(true, || parser.help_text());

        prop_assert!(contains_dim(&help, "(required)"),
            "(required) should be dim");
        prop_assert!(contains_dim(&help, &format!("[default: {default_val}]")),
            "[default: {}] should be dim", default_val);
        prop_assert!(contains_dim(&help, &format!("[env: {env_var}]")),
            "[env: {}] should be dim", env_var);
    }
}

// Section headers are bold yellow
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop4_section_headers_bold_yellow(
        flag in arb_safe_flag_def(),
        positional in arb_safe_positional_def(),
    ) {
        let (long, short, desc) = flag;
        let s = short.filter(|&c| c != 'h');
        let (pos_name, pos_desc, pos_required) = positional;

        let mut f = Flag::new(&long).desc(&desc);
        if let Some(ch) = s { f = f.short(ch); }
        let mut builder = ArgBuilder::new()
            .name("testapp")
            .flag(f);
        let mut p = Pos::new(&pos_name).desc(&pos_desc);
        if pos_required { p = p.required(); }
        builder = builder.positional(p);

        let parser = builder.build().unwrap();
        let help = nanocolor::with_colors_override(true, || parser.help_text());

        prop_assert!(contains_bold_yellow(&help, "Options:"),
            "Options: header should be bold yellow");
        prop_assert!(contains_bold_yellow(&help, "Positional arguments:"),
            "Positional arguments: header should be bold yellow");
    }
}

// Positional names in usage line are cyan
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop5_positional_names_in_usage_cyan(
        positionals in prop::collection::vec(arb_safe_positional_def(), 1..4),
    ) {
        let mut builder = ArgBuilder::new().name("testapp");
        for (name, desc, required) in &positionals {
            let mut p = Pos::new(name).desc(desc);
            if *required { p = p.required(); }
            builder = builder.positional(p);
        }

        let parser = builder.build().unwrap();
        let help = nanocolor::with_colors_override(true, || parser.help_text());
        let usage_line = help.lines().find(|l| strip_ansi(l).starts_with("Usage:")).unwrap();

        for (name, _, required) in &positionals {
            if *required {
                prop_assert!(contains_cyan(usage_line, &format!("<{name}>")),
                    "Required positional <{}> should be cyan in usage line", name);
            } else {
                prop_assert!(contains_cyan(usage_line, &format!("[{name}]")),
                    "Optional positional [{}] should be cyan in usage line", name);
            }
        }
    }
}

// Error messages have bold red prefix and yellow argument
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop6_error_messages_bold_red_prefix_yellow_arg(
        arg_name in arb_safe_identifier(),
    ) {
        use nanoargs::ParseError;

        let variants_with_yellow_name: Vec<ParseError> = vec![
            ParseError::MissingRequired(arg_name.clone()),
            ParseError::UnknownArgument(arg_name.clone()),
            ParseError::UnknownSubcommand(arg_name.clone()),
        ];

        for err in &variants_with_yellow_name {
            let msg = nanocolor::with_colors_override(true, || format!("{err}"));
            prop_assert!(contains_bold_red(&msg, "error:"),
                "error: prefix should be bold red in {:?}", msg);
            prop_assert!(contains_yellow(&msg, &arg_name),
                "argument '{}' should be yellow in {:?}", arg_name, msg);
        }

        let missing_val = ParseError::MissingValue(arg_name.clone());
        let msg = nanocolor::with_colors_override(true, || format!("{missing_val}"));
        prop_assert!(contains_bold_red(&msg, "error:"),
            "error: prefix should be bold red for MissingValue");
        prop_assert!(contains_yellow(&msg, &format!("--{arg_name}")),
            "--{} should be yellow for MissingValue", arg_name);

        let dup_opt = ParseError::DuplicateOption(arg_name.clone());
        let msg = nanocolor::with_colors_override(true, || format!("{dup_opt}"));
        prop_assert!(contains_bold_red(&msg, "error:"),
            "error: prefix should be bold red for DuplicateOption");
        prop_assert!(contains_yellow(&msg, &format!("--{arg_name}")),
            "--{} should be yellow for DuplicateOption", arg_name);

        let variants_no_yellow: Vec<ParseError> = vec![
            ParseError::InvalidFormat(arg_name.clone()),
            ParseError::NoSubcommand(arg_name.clone()),
        ];

        for err in &variants_no_yellow {
            let msg = nanocolor::with_colors_override(true, || format!("{err}"));
            prop_assert!(contains_bold_red(&msg, "error:"),
                "error: prefix should be bold red in {:?}", msg);
        }

        let help_err = ParseError::HelpRequested("some help text".to_string());
        let help_msg = nanocolor::with_colors_override(true, || format!("{help_err}"));
        prop_assert_eq!(help_msg, "some help text");

        let ver_err = ParseError::VersionRequested("v1.0.0".to_string());
        let ver_msg = nanocolor::with_colors_override(true, || format!("{ver_err}"));
        prop_assert_eq!(ver_msg, "v1.0.0");
    }
}
