mod common;

use common::*;
use nanoargs::{ArgBuilder, Flag, Opt, ParseError, Pos};
use proptest::prelude::*;

// ParseError Display is non-empty for all variants
proptest! {
    #[test]
    fn prop10_parse_error_display_is_non_empty(
        msg in "[a-zA-Z0-9_ ]{1,50}"
    ) {
        let variants = vec![
            ParseError::MissingRequired(msg.clone()),
            ParseError::MissingValue(msg.clone()),
            ParseError::UnknownArgument(msg.clone()),
            ParseError::InvalidFormat(msg.clone()),
            ParseError::HelpRequested(msg.clone()),
            ParseError::DuplicateOption(msg.clone()),
        ];
        for variant in variants {
            let display = variant.to_string();
            prop_assert!(!display.is_empty(), "Display was empty for {:?}", variant);
        }
    }
}

// Builder preserves all argument definitions
proptest! {
    #[test]
    fn prop1_builder_preserves_all_definitions(
        flags in prop::collection::vec(arb_flag_def(), 0..5),
        options in prop::collection::vec(arb_option_def(), 0..5),
        positionals in prop::collection::vec(arb_positional_def(), 0..5),
        prog_name in prop::option::of(arb_identifier()),
        prog_desc in prop::option::of(arb_description()),
    ) {
        {
            let mut longs = std::collections::HashSet::new();
            let mut shorts = std::collections::HashSet::new();
            for (long, short, _) in &flags {
                prop_assume!(longs.insert(long.clone()));
                if let Some(ch) = short { prop_assume!(shorts.insert(*ch)); }
            }
            for (long, short, _, _, _, _) in &options {
                prop_assume!(longs.insert(long.clone()));
                if let Some(ch) = short { prop_assume!(shorts.insert(*ch)); }
            }
        }

        let mut builder = ArgBuilder::new();
        if let Some(ref n) = prog_name { builder = builder.name(n); }
        if let Some(ref d) = prog_desc { builder = builder.description(d); }

        for (long, short, desc) in &flags {
            let mut f = Flag::new(long).desc(desc);
            if let Some(ch) = short { f = f.short(*ch); }
            builder = builder.flag(f);
        }
        for (long, short, placeholder, desc, required, default) in &options {
            let mut o = Opt::new(long).placeholder(placeholder).desc(desc);
            if let Some(ch) = short { o = o.short(*ch); }
            if *required {
                o = o.required();
            } else if let Some(def) = default {
                o = o.default(def);
            }
            builder = builder.option(o);
        }
        for (name, desc, required) in &positionals {
            let mut p = Pos::new(name).desc(desc);
            if *required { p = p.required(); }
            builder = builder.positional(p);
        }

        let parser = builder.build().unwrap();

        prop_assert_eq!(parser.flags().len(), flags.len());
        for (i, (long, short, desc)) in flags.iter().enumerate() {
            prop_assert_eq!(&parser.flags()[i].long, long);
            prop_assert_eq!(&parser.flags()[i].short, short);
            prop_assert_eq!(&parser.flags()[i].description, desc);
        }

        prop_assert_eq!(parser.options().len(), options.len());
        for (i, (long, short, placeholder, desc, required, default)) in options.iter().enumerate() {
            prop_assert_eq!(&parser.options()[i].long, long);
            prop_assert_eq!(&parser.options()[i].short, short);
            prop_assert_eq!(&parser.options()[i].placeholder, placeholder);
            prop_assert_eq!(&parser.options()[i].description, desc);
            prop_assert_eq!(parser.options()[i].required, *required);
            if *required {
                prop_assert_eq!(&parser.options()[i].default, &None);
            } else {
                prop_assert_eq!(&parser.options()[i].default, default);
            }
        }

        prop_assert_eq!(parser.positionals().len(), positionals.len());
        for (i, (name, desc, required)) in positionals.iter().enumerate() {
            prop_assert_eq!(&parser.positionals()[i].name, name);
            prop_assert_eq!(&parser.positionals()[i].description, desc);
            prop_assert_eq!(parser.positionals()[i].required, *required);
        }

        prop_assert_eq!(parser.program_name(), prog_name.as_deref());
        prop_assert_eq!(parser.program_desc(), prog_desc.as_deref());
    }
}

// Flag presence determines boolean result
proptest! {
    #[test]
    fn prop2_flag_presence_determines_boolean(
        flag_long in arb_identifier().prop_filter("avoid help collision", |s| s != "help"),
        flag_short in any::<u32>().prop_map(|v| {
            let c = (b'a' + (v % 26) as u8) as char;
            if c == 'h' { 'z' } else { c }
        }),
        present in any::<bool>(),
        use_short in any::<bool>(),
    ) {
        let parser = ArgBuilder::new()
            .flag(Flag::new(&flag_long).desc("test flag").short(flag_short))
            .build().unwrap();

        let args: Vec<String> = if present {
            if use_short {
                vec![format!("-{flag_short}")]
            } else {
                vec![format!("--{flag_long}")]
            }
        } else {
            vec![]
        };

        let result = parser.parse(args).unwrap();
        prop_assert_eq!(result.get_flag(&flag_long), present);
    }
}

// Option value parsing across syntax variants
proptest! {
    #[test]
    fn prop3_option_syntax_variants(
        opt_long in arb_identifier().prop_filter("avoid help collision", |s| s != "help"),
        opt_short in any::<u32>().prop_map(|v| {
            let c = (b'a' + (v % 26) as u8) as char;
            if c == 'h' { 'z' } else { c }
        }),
        value in "[a-zA-Z0-9]{1,20}",
        variant in 0u32..4,
    ) {
        let parser = ArgBuilder::new()
            .option(Opt::new(&opt_long).placeholder("VAL").desc("test option").short(opt_short))
            .build().unwrap();

        let args: Vec<String> = match variant {
            0 => vec![format!("--{opt_long}"), value.clone()],
            1 => vec![format!("--{opt_long}={value}")],
            2 => vec![format!("-{opt_short}"), value.clone()],
            3 => vec![format!("-{opt_short}={value}")],
            _ => unreachable!(),
        };

        let result = parser.parse(args).unwrap();
        prop_assert_eq!(result.get_option(&opt_long), Some(value.as_str()));
    }
}

// Positional arguments collected in order
proptest! {
    #[test]
    fn prop4_positional_ordering(
        positionals in prop::collection::vec("[a-zA-Z0-9]{1,10}", 1..6),
        use_separator in any::<bool>(),
    ) {
        let mut builder = ArgBuilder::new()
            .flag(Flag::new("verbose").desc("verbose").short('v'));

        for i in 0..positionals.len() {
            builder = builder.positional(Pos::new(&format!("pos{i}")).desc("positional arg"));
        }

        let parser = builder.build().unwrap();

        let mut args: Vec<String> = Vec::new();
        if use_separator {
            args.push("--".to_string());
        }
        args.extend(positionals.iter().cloned());

        let result = parser.parse(args).unwrap();
        let parsed = result.get_positionals();
        prop_assert_eq!(parsed.len(), positionals.len());
        for (i, val) in positionals.iter().enumerate() {
            prop_assert_eq!(&parsed[i], val);
        }
    }
}

// Missing/unknown arguments produce correct errors
proptest! {
    #[test]
    fn prop7_missing_required_option_produces_error(
        opt_long in arb_identifier(),
    ) {
        let parser = ArgBuilder::new()
            .option(Opt::new(&opt_long).placeholder("VAL").desc("required opt").required())
            .build().unwrap();
        let result = parser.parse(vec![]);
        prop_assert_eq!(result, Err(ParseError::MissingRequired(opt_long)));
    }

    #[test]
    fn prop7b_missing_required_positional_produces_error(
        pos_name in arb_identifier(),
    ) {
        let parser = ArgBuilder::new()
            .positional(Pos::new(&pos_name).desc("required pos").required())
            .build().unwrap();
        let result = parser.parse(vec![]);
        prop_assert_eq!(result, Err(ParseError::MissingRequired(pos_name)));
    }

    #[test]
    fn prop8_missing_option_value_produces_error(
        opt_long in arb_identifier(),
    ) {
        let parser = ArgBuilder::new()
            .option(Opt::new(&opt_long).placeholder("VAL").desc("test opt"))
            .build().unwrap();
        let args = vec![format!("--{opt_long}")];
        let result = parser.parse(args);
        prop_assert_eq!(result, Err(ParseError::MissingValue(opt_long)));
    }

    #[test]
    fn prop9_unknown_arguments_produce_errors(
        unknown in arb_identifier(),
    ) {
        let parser = ArgBuilder::new()
            .flag(Flag::new("known").desc("a known flag"))
            .build().unwrap();
        prop_assume!(unknown != "known");
        let token = format!("--{unknown}");
        let result = parser.parse(vec![token.clone()]);
        prop_assert_eq!(result, Err(ParseError::UnknownArgument(token)));
    }
}

// Typed option parsing via FromStr
proptest! {
    #[test]
    fn prop5_typed_option_parsing_via_fromstr(
        opt_long in arb_identifier().prop_filter("avoid help collision", |s| s != "help"),
        value in -10000i64..10000i64,
    ) {
        let parser = ArgBuilder::new()
            .option(Opt::new(&opt_long).placeholder("NUM").desc("a numeric option"))
            .build().unwrap();
        let args = vec![format!("--{opt_long}"), value.to_string()];
        let result = parser.parse(args).unwrap();
        let parsed: Option<Result<i64, _>> = result.get_option_parsed(&opt_long);
        prop_assert!(parsed.is_some());
        prop_assert_eq!(parsed.unwrap().unwrap(), value);
    }

    #[test]
    fn prop5b_typed_option_parsing_invalid_returns_err(
        opt_long in arb_identifier().prop_filter("avoid help collision", |s| s != "help"),
        value in "[a-zA-Z]{1,10}",
    ) {
        let parser = ArgBuilder::new()
            .option(Opt::new(&opt_long).placeholder("NUM").desc("a numeric option"))
            .build().unwrap();
        let args = vec![format!("--{opt_long}"), value.clone()];
        let result = parser.parse(args).unwrap();
        let parsed: Option<Result<i64, _>> = result.get_option_parsed(&opt_long);
        prop_assert!(parsed.is_some());
        prop_assert!(parsed.unwrap().is_err());
    }
}

// Default values for missing options
proptest! {
    #[test]
    fn prop6_default_values_for_missing_options(
        opt_long in arb_identifier().prop_filter("avoid help collision", |s| s != "help"),
        default_val in "[a-zA-Z0-9]{1,20}",
    ) {
        let parser = ArgBuilder::new()
            .option(Opt::new(&opt_long).placeholder("VAL").desc("opt with default").default(&default_val))
            .build().unwrap();
        let result = parser.parse(vec![]).unwrap();
        prop_assert_eq!(result.get_option(&opt_long), Some(default_val.as_str()));
    }

    #[test]
    fn prop6b_no_default_returns_none(
        opt_long in arb_identifier().prop_filter("avoid help collision", |s| s != "help"),
    ) {
        let parser = ArgBuilder::new()
            .option(Opt::new(&opt_long).placeholder("VAL").desc("opt without default"))
            .build().unwrap();
        let result = parser.parse(vec![]).unwrap();
        prop_assert_eq!(result.get_option(&opt_long), None);
    }
}

// Help text contains all registered definitions
proptest! {
    #[test]
    fn prop12_help_text_contains_all_definitions(
        num_flags in 0usize..4,
        num_options in 0usize..4,
        num_positionals in 0usize..4,
        prog_name in prop::option::of(arb_identifier()),
        prog_desc in prop::option::of(arb_description()),
        flag_shorts in prop::collection::vec(arb_short(), 4),
        flag_descs in prop::collection::vec(arb_description(), 4),
        opt_shorts in prop::collection::vec(arb_short(), 4),
        opt_descs in prop::collection::vec(arb_description(), 4),
        opt_required in prop::collection::vec(any::<bool>(), 4),
        pos_descs in prop::collection::vec(arb_description(), 4),
        pos_required in prop::collection::vec(any::<bool>(), 4),
    ) {
        {
            let mut shorts = std::collections::HashSet::new();
            for i in 0..num_flags {
                if let Some(ch) = flag_shorts[i] { prop_assume!(shorts.insert(ch)); }
            }
            for i in 0..num_options {
                if let Some(ch) = opt_shorts[i] { prop_assume!(shorts.insert(ch)); }
            }
        }

        let mut builder = ArgBuilder::new();
        if let Some(ref n) = prog_name { builder = builder.name(n); }
        if let Some(ref d) = prog_desc { builder = builder.description(d); }

        let flag_names: Vec<String> = (0..num_flags).map(|i| format!("flg{i}xx")).collect();
        let opt_names: Vec<String> = (0..num_options).map(|i| format!("opt{i}xx")).collect();
        let opt_phs: Vec<String> = (0..num_options).map(|i| format!("PH{i}XX")).collect();
        let pos_names: Vec<String> = (0..num_positionals).map(|i| format!("pos{i}xx")).collect();

        for i in 0..num_flags {
            let mut f = Flag::new(&flag_names[i]).desc(&flag_descs[i]);
            if let Some(ch) = flag_shorts[i] { f = f.short(ch); }
            builder = builder.flag(f);
        }
        for i in 0..num_options {
            let mut o = Opt::new(&opt_names[i]).placeholder(&opt_phs[i]).desc(&opt_descs[i]);
            if let Some(ch) = opt_shorts[i] { o = o.short(ch); }
            if opt_required[i] { o = o.required(); }
            builder = builder.option(o);
        }
        for i in 0..num_positionals {
            let mut p = Pos::new(&pos_names[i]).desc(&pos_descs[i]);
            if pos_required[i] { p = p.required(); }
            builder = builder.positional(p);
        }

        let parser = builder.build().unwrap();
        let help_raw = parser.help_text();
        // Strip ANSI codes so assertions work with or without the color feature
        let help = strip_ansi_inline(&help_raw);

        let expected_name = prog_name.as_deref().unwrap_or("program");
        let usage_str = format!("Usage: {}", expected_name);
        prop_assert!(help.contains(&usage_str));

        if let Some(ref d) = prog_desc {
            prop_assert!(help.contains(d));
        }

        for i in 0..num_flags {
            let flag_str = format!("--{}", flag_names[i]);
            prop_assert!(help.contains(&flag_str));
            prop_assert!(help.contains(flag_descs[i].as_str()));
        }

        for i in 0..num_options {
            let opt_line = help.lines()
                .find(|l| l.contains(&format!("--{}", opt_names[i])))
                .unwrap();
            prop_assert!(opt_line.contains(&opt_phs[i]));
            if opt_required[i] {
                prop_assert!(opt_line.contains("required"));
            }
        }

        for i in 0..num_positionals {
            prop_assert!(help.contains(&pos_names[i]));
            if pos_required[i] {
                let pos_line = help.lines()
                    .find(|l| l.starts_with("  ") && l.contains(pos_names[i].as_str()))
                    .unwrap();
                prop_assert!(pos_line.contains("required"));
            }
        }
    }
}

// Help flag returns help text
proptest! {
    #[test]
    fn prop11_help_flag_returns_help_text(
        flags in prop::collection::vec(arb_flag_def(), 0..3),
        options in prop::collection::vec(arb_option_def(), 0..3),
        use_short in any::<bool>(),
        inject_pos in 0usize..10,
    ) {
        {
            let mut longs = std::collections::HashSet::new();
            let mut shorts = std::collections::HashSet::new();
            for (long, short, _) in &flags {
                prop_assume!(longs.insert(long.clone()));
                if let Some(ch) = short { prop_assume!(shorts.insert(*ch)); }
            }
            for (long, short, _, _, _, _) in &options {
                prop_assume!(longs.insert(long.clone()));
                if let Some(ch) = short { prop_assume!(shorts.insert(*ch)); }
            }
        }

        let mut builder = ArgBuilder::new().name("testprog");
        for (long, short, desc) in &flags {
            let mut f = Flag::new(long).desc(desc);
            if let Some(ch) = short { f = f.short(*ch); }
            builder = builder.flag(f);
        }
        for (long, short, placeholder, desc, _required, default) in &options {
            let mut o = Opt::new(long).placeholder(placeholder).desc(desc);
            if let Some(ch) = short { o = o.short(*ch); }
            if let Some(def) = default { o = o.default(def); }
            builder = builder.option(o);
        }

        let parser = builder.build().unwrap();
        let expected_help = parser.help_text();

        let help_token = if use_short { "-h".to_string() } else { "--help".to_string() };
        let mut args: Vec<String> = Vec::new();
        let pos = inject_pos % 5;
        for i in 0..pos {
            args.push(format!("arg{i}"));
        }
        args.push(help_token);

        let result = parser.parse(args);
        match result {
            Err(ParseError::HelpRequested(text)) => {
                prop_assert_eq!(text, expected_help);
            }
            other => {
                prop_assert!(false, "Expected HelpRequested, got {:?}", other);
            }
        }
    }
}
