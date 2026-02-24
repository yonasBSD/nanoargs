mod common;

use common::*;
use nanoargs::{ArgBuilder, Opt, ParseError};
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
