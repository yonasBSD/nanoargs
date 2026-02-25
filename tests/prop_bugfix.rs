mod common;

use common::*;
use nanoargs::{ArgBuilder, Opt, OptionError, ParseError, ParseResultBuilder};
use proptest::prelude::*;

// Missing required global option with subcommand returns MissingRequired
proptest! {
    #[test]
    fn prop_bugfix_missing_required_global_option_with_subcommand(
        opt_name in arb_identifier().prop_filter("avoid help collision", |s| s != "help"),
        subcmd_name in arb_subcommand_name(),
    ) {
        let sub_parser = ArgBuilder::new().build().unwrap();

        let parent = ArgBuilder::new()
            .option(Opt::new(&opt_name).placeholder("VAL").desc("a required global option").required())
            .subcommand(&subcmd_name, "test subcommand", sub_parser)
            .build()
            .unwrap();

        let args = vec![subcmd_name.clone()];
        let result = parent.parse(args);

        match result {
            Err(nanoargs::ParseError::MissingRequired(name)) => {
                prop_assert_eq!(name, opt_name);
            }
            other => {
                prop_assert!(false, "Expected Err(MissingRequired(\"{}\")), got {:?}", opt_name, other);
            }
        }
    }
}

// Providing all required global options with subcommand parses successfully
proptest! {
    #[test]
    fn prop_bugfix_required_global_options_provided_parses_ok(
        req_opt_name in arb_identifier().prop_filter("avoid help collision", |s| s != "help"),
        opt_opt_name in arb_identifier().prop_filter("avoid help collision", |s| s != "help"),
        subcmd_name in arb_subcommand_name(),
        value in "[a-zA-Z0-9]{1,20}",
    ) {
        prop_assume!(req_opt_name != opt_opt_name);

        let sub_parser = ArgBuilder::new().build().unwrap();

        let parent = ArgBuilder::new()
            .option(Opt::new(&req_opt_name).placeholder("VAL").desc("a required global option").required())
            .option(Opt::new(&opt_opt_name).placeholder("VAL").desc("an optional global option"))
            .subcommand(&subcmd_name, "test subcommand", sub_parser)
            .build()
            .unwrap();

        let args = vec![
            format!("--{req_opt_name}"),
            value.clone(),
            subcmd_name.clone(),
        ];
        let result = parent.parse(args);

        let parsed = result.unwrap();
        prop_assert_eq!(parsed.get_option(&req_opt_name), Some(value.as_str()));
        prop_assert_eq!(parsed.get_option(&opt_opt_name), None);
        prop_assert_eq!(parsed.subcommand(), Some(subcmd_name.as_str()));
    }
}

// Consistency between parsing paths for required option validation
proptest! {
    #[test]
    fn prop_bugfix_consistency_between_parsing_paths(
        opt_name in arb_identifier().prop_filter("avoid help collision", |s| s != "help"),
        subcmd_name in arb_subcommand_name(),
    ) {
        let no_sub = ArgBuilder::new()
            .option(Opt::new(&opt_name).placeholder("VAL").desc("a required option").required())
            .build()
            .unwrap();

        let sub_parser = ArgBuilder::new().build().unwrap();
        let with_sub = ArgBuilder::new()
            .option(Opt::new(&opt_name).placeholder("VAL").desc("a required option").required())
            .subcommand(&subcmd_name, "test subcommand", sub_parser)
            .build()
            .unwrap();

        let result_no_sub = no_sub.parse(vec![]);
        let result_with_sub = with_sub.parse(vec![subcmd_name.clone()]);

        match (&result_no_sub, &result_with_sub) {
            (
                Err(ParseError::MissingRequired(name1)),
                Err(ParseError::MissingRequired(name2)),
            ) => {
                prop_assert_eq!(name1, name2);
                prop_assert_eq!(name1, &opt_name);
            }
            _ => {
                prop_assert!(
                    false,
                    "Expected both to be Err(MissingRequired), got no_sub={:?}, with_sub={:?}",
                    result_no_sub, result_with_sub
                );
            }
        }
    }
}

// Feature: or-default-error-propagation, Property 3: get_option_or_default returns Err(ParseFailed) on unparseable value
// **Validates: Requirements 1.3**
proptest! {
    #[test]
    fn prop_bugfix_get_option_or_default_err_on_unparseable(
        name in arb_identifier(),
        bad_val in arb_non_numeric_string(),
        default_val in any::<u32>(),
    ) {
        let result = ParseResultBuilder::new()
            .option(&name, &bad_val)
            .build();
        let actual = result.get_option_or_default::<u32>(&name, default_val);
        match actual {
            Err(OptionError::ParseFailed { option, message }) => {
                prop_assert_eq!(option, name);
                prop_assert!(!message.is_empty(), "parse failure message should be non-empty");
            }
            other => prop_assert!(false, "expected Err(ParseFailed), got {:?}", other),
        }
    }
}

// Feature: or-default-error-propagation, Property 6: get_option_or returns Err(ParseFailed) without calling closure on unparseable value
// **Validates: Requirements 2.3**
proptest! {
    #[test]
    fn prop_bugfix_get_option_or_err_no_closure_on_unparseable(
        name in arb_identifier(),
        bad_val in arb_non_numeric_string(),
        fallback in any::<u32>(),
    ) {
        let result = ParseResultBuilder::new()
            .option(&name, &bad_val)
            .build();
        let closure_called = std::cell::Cell::new(false);
        let actual = result.get_option_or::<u32, _>(&name, || {
            closure_called.set(true);
            fallback
        });
        match actual {
            Err(OptionError::ParseFailed { option, message }) => {
                prop_assert_eq!(option, name);
                prop_assert!(!message.is_empty(), "parse failure message should be non-empty");
            }
            other => prop_assert!(false, "expected Err(ParseFailed), got {:?}", other),
        }
        prop_assert!(!closure_called.get(), "closure should not be called when option is present but unparseable");
    }
}

// Feature: or-default-error-propagation, Property 9: get_option_values_or_default returns Err(ParseFailed) when any value fails to parse
// **Validates: Requirements 3.3**
proptest! {
    #[test]
    fn prop_bugfix_get_option_values_or_default_err_on_any_parse_failure(
        name in arb_identifier(),
        default_values in prop::collection::vec(any::<u32>(), 0..=5),
        prefix_valid in prop::collection::vec(any::<u32>(), 0..=3),
        bad_val in arb_non_numeric_string(),
        suffix_valid in prop::collection::vec(any::<u32>(), 0..=3),
    ) {
        let mut builder = ParseResultBuilder::new();
        for v in &prefix_valid {
            builder = builder.multi_option(&name, &v.to_string());
        }
        builder = builder.multi_option(&name, &bad_val);
        for v in &suffix_valid {
            builder = builder.multi_option(&name, &v.to_string());
        }
        let result = builder.build();
        let actual = result.get_option_values_or_default::<u32>(&name, default_values);
        match actual {
            Err(OptionError::ParseFailed { option, message }) => {
                prop_assert_eq!(option, name);
                prop_assert!(!message.is_empty(), "parse failure message should be non-empty");
            }
            other => prop_assert!(false, "expected Err(ParseFailed), got {:?}", other),
        }
    }
}
