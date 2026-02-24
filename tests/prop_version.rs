mod common;

use common::*;
use nanoargs::{ArgBuilder, Flag, ParseError};
use proptest::prelude::*;

/// Arbitrary version string: typical semver-like characters.
fn arb_version_string() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9][a-zA-Z0-9.\\-+]{0,19}"
}

// Build-time conflict detection for -V
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop7_build_rejects_short_v_flag_when_version_set(
        version in arb_version_string(),
        long in arb_identifier(),
    ) {
        let result = ArgBuilder::new()
            .version(&version)
            .flag(Flag::new(&long).desc("desc").short('V'))
            .build();
        match result {
            Err(ParseError::InvalidFormat(msg)) => {
                prop_assert!(msg.contains("-V"), "error should mention -V: {}", msg);
            }
            other => prop_assert!(false, "expected InvalidFormat, got {:?}", other),
        }
    }

    #[test]
    fn prop7_build_rejects_short_v_option_when_version_set(
        version in arb_version_string(),
        long in arb_identifier(),
    ) {
        let result = ArgBuilder::new()
            .version(&version)
            .option(nanoargs::Opt::new(&long).placeholder("VAL").desc("desc").short('V'))
            .build();
        match result {
            Err(ParseError::InvalidFormat(msg)) => {
                prop_assert!(msg.contains("-V"), "error should mention -V: {}", msg);
            }
            other => prop_assert!(false, "expected InvalidFormat, got {:?}", other),
        }
    }

    #[test]
    fn prop7_build_allows_short_v_without_version(
        long in arb_identifier(),
    ) {
        let result = ArgBuilder::new()
            .flag(Flag::new(&long).desc("desc").short('V'))
            .build();
        prop_assert!(result.is_ok(), "expected Ok, got {:?}", result);
    }
}

/// Arbitrary optional program name.
fn arb_program_name() -> impl Strategy<Value = Option<String>> {
    prop_oneof![Just(None), arb_identifier().prop_map(Some),]
}

// Version flag triggers VersionRequested
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop1_version_long_flag_triggers_version_requested(
        version in arb_version_string(),
        name in arb_program_name(),
    ) {
        let mut builder = ArgBuilder::new().version(&version);
        if let Some(ref n) = name {
            builder = builder.name(n);
        }
        let parser = builder.build().unwrap();
        let result = parser.parse(vec!["--version".to_string()]);
        let expected = parser.version_text().unwrap();
        match result {
            Err(ParseError::VersionRequested(text)) => {
                prop_assert_eq!(text, expected);
            }
            other => prop_assert!(false, "expected VersionRequested, got {:?}", other),
        }
    }

    #[test]
    fn prop1_version_short_flag_triggers_version_requested(
        version in arb_version_string(),
        name in arb_program_name(),
    ) {
        let mut builder = ArgBuilder::new().version(&version);
        if let Some(ref n) = name {
            builder = builder.name(n);
        }
        let parser = builder.build().unwrap();
        let result = parser.parse(vec!["-V".to_string()]);
        let expected = parser.version_text().unwrap();
        match result {
            Err(ParseError::VersionRequested(text)) => {
                prop_assert_eq!(text, expected);
            }
            other => prop_assert!(false, "expected VersionRequested, got {:?}", other),
        }
    }
}

// No version means unknown argument
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop2_no_version_long_flag_returns_unknown(
        name in arb_program_name(),
    ) {
        let mut builder = ArgBuilder::new();
        if let Some(ref n) = name {
            builder = builder.name(n);
        }
        let parser = builder.build().unwrap();
        let result = parser.parse(vec!["--version".to_string()]);
        match result {
            Err(ParseError::UnknownArgument(token)) => {
                prop_assert_eq!(token, "--version");
            }
            other => prop_assert!(false, "expected UnknownArgument, got {:?}", other),
        }
    }

    #[test]
    fn prop2_no_version_short_flag_returns_unknown(
        name in arb_program_name(),
    ) {
        let mut builder = ArgBuilder::new();
        if let Some(ref n) = name {
            builder = builder.name(n);
        }
        let parser = builder.build().unwrap();
        let result = parser.parse(vec!["-V".to_string()]);
        match result {
            Err(ParseError::UnknownArgument(token)) => {
                prop_assert_eq!(token, "-V");
            }
            other => prop_assert!(false, "expected UnknownArgument, got {:?}", other),
        }
    }
}

// Version text formatting
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop3_version_text_with_name(
        version in arb_version_string(),
        name in arb_identifier(),
    ) {
        let parser = ArgBuilder::new()
            .name(&name)
            .version(&version)
            .build()
            .unwrap();
        let text = parser.version_text();
        prop_assert_eq!(text, Some(format!("{name} {version}")));
    }

    #[test]
    fn prop3_version_text_without_name(
        version in arb_version_string(),
    ) {
        let parser = ArgBuilder::new()
            .version(&version)
            .build()
            .unwrap();
        let text = parser.version_text();
        prop_assert_eq!(text, Some(version));
    }

    #[test]
    fn prop3_version_text_none_when_no_version(
        name in arb_program_name(),
    ) {
        let mut builder = ArgBuilder::new();
        if let Some(ref n) = name {
            builder = builder.name(n);
        }
        let parser = builder.build().unwrap();
        prop_assert_eq!(parser.version_text(), None);
    }
}

// First-flag-wins for help vs version
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop4_help_first_returns_help(
        version in arb_version_string(),
        name in arb_program_name(),
    ) {
        let mut builder = ArgBuilder::new().version(&version);
        if let Some(ref n) = name {
            builder = builder.name(n);
        }
        let parser = builder.build().unwrap();
        let result = parser.parse(vec!["--help".into(), "--version".into()]);
        match result {
            Err(ParseError::HelpRequested(_)) => {}
            other => prop_assert!(false, "expected HelpRequested, got {:?}", other),
        }
    }

    #[test]
    fn prop4_version_first_returns_version(
        version in arb_version_string(),
        name in arb_program_name(),
    ) {
        let mut builder = ArgBuilder::new().version(&version);
        if let Some(ref n) = name {
            builder = builder.name(n);
        }
        let parser = builder.build().unwrap();
        let expected = parser.version_text().unwrap();
        let result = parser.parse(vec!["--version".into(), "--help".into()]);
        match result {
            Err(ParseError::VersionRequested(text)) => {
                prop_assert_eq!(text, expected);
            }
            other => prop_assert!(false, "expected VersionRequested, got {:?}", other),
        }
    }

    #[test]
    fn prop4_short_v_first_returns_version(
        version in arb_version_string(),
        name in arb_program_name(),
    ) {
        let mut builder = ArgBuilder::new().version(&version);
        if let Some(ref n) = name {
            builder = builder.name(n);
        }
        let parser = builder.build().unwrap();
        let expected = parser.version_text().unwrap();
        let result = parser.parse(vec!["-V".into(), "-h".into()]);
        match result {
            Err(ParseError::VersionRequested(text)) => {
                prop_assert_eq!(text, expected);
            }
            other => prop_assert!(false, "expected VersionRequested, got {:?}", other),
        }
    }

    #[test]
    fn prop4_short_h_first_returns_help(
        version in arb_version_string(),
        name in arb_program_name(),
    ) {
        let mut builder = ArgBuilder::new().version(&version);
        if let Some(ref n) = name {
            builder = builder.name(n);
        }
        let parser = builder.build().unwrap();
        let result = parser.parse(vec!["-h".into(), "-V".into()]);
        match result {
            Err(ParseError::HelpRequested(_)) => {}
            other => prop_assert!(false, "expected HelpRequested, got {:?}", other),
        }
    }
}

// Combined token version interception
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop5_combined_token_with_v_triggers_version(
        version in arb_version_string(),
        name in arb_program_name(),
        prefix_flags in arb_flag_shorts(1..=3),
    ) {
        let mut builder = ArgBuilder::new().version(&version);
        if let Some(ref n) = name {
            builder = builder.name(n);
        }
        for (i, &ch) in prefix_flags.iter().enumerate() {
            let long = format!("flag{}", i);
            builder = builder.flag(Flag::new(&long).desc("test flag").short(ch));
        }
        let parser = builder.build().unwrap();
        let expected = parser.version_text().unwrap();

        let mut token = String::from("-");
        for &ch in &prefix_flags {
            token.push(ch);
        }
        token.push('V');

        let result = parser.parse(vec![token]);
        match result {
            Err(ParseError::VersionRequested(text)) => {
                prop_assert_eq!(text, expected);
            }
            other => prop_assert!(false, "expected VersionRequested, got {:?}", other),
        }
    }
}

// Subcommand version scoping
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop6_version_before_subcommand_returns_parent(
        parent_version in arb_version_string(),
        sub_version in arb_version_string(),
        parent_name in arb_program_name(),
    ) {
        let sub_parser = ArgBuilder::new()
            .version(&sub_version)
            .build()
            .unwrap();
        let mut builder = ArgBuilder::new().version(&parent_version);
        if let Some(ref n) = parent_name {
            builder = builder.name(n);
        }
        let parser = builder
            .subcommand("sub", "a subcommand", sub_parser)
            .build()
            .unwrap();
        let expected = parser.version_text().unwrap();

        let result = parser.parse(vec!["--version".into(), "sub".into()]);
        match result {
            Err(ParseError::VersionRequested(text)) => {
                prop_assert_eq!(text, expected);
            }
            other => prop_assert!(false, "expected parent VersionRequested, got {:?}", other),
        }
    }

    #[test]
    fn prop6_version_after_subcommand_returns_sub(
        parent_version in arb_version_string(),
        sub_version in arb_version_string(),
        parent_name in arb_program_name(),
    ) {
        let sub_parser = ArgBuilder::new()
            .version(&sub_version)
            .build()
            .unwrap();
        let mut builder = ArgBuilder::new().version(&parent_version);
        if let Some(ref n) = parent_name {
            builder = builder.name(n);
        }
        let parser = builder
            .subcommand("sub", "a subcommand", sub_parser)
            .build()
            .unwrap();

        let result = parser.parse(vec!["sub".into(), "--version".into()]);
        match result {
            Err(ParseError::VersionRequested(text)) => {
                prop_assert_eq!(text, sub_version);
            }
            other => prop_assert!(false, "expected sub VersionRequested, got {:?}", other),
        }
    }

    #[test]
    fn prop6_short_v_before_subcommand_returns_parent(
        parent_version in arb_version_string(),
        sub_version in arb_version_string(),
        parent_name in arb_program_name(),
    ) {
        let sub_parser = ArgBuilder::new()
            .version(&sub_version)
            .build()
            .unwrap();
        let mut builder = ArgBuilder::new().version(&parent_version);
        if let Some(ref n) = parent_name {
            builder = builder.name(n);
        }
        let parser = builder
            .subcommand("sub", "a subcommand", sub_parser)
            .build()
            .unwrap();
        let expected = parser.version_text().unwrap();

        let result = parser.parse(vec!["-V".into(), "sub".into()]);
        match result {
            Err(ParseError::VersionRequested(text)) => {
                prop_assert_eq!(text, expected);
            }
            other => prop_assert!(false, "expected parent VersionRequested, got {:?}", other),
        }
    }
}
