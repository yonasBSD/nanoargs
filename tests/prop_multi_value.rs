mod common;

use common::*;
use nanoargs::{ArgBuilder, Opt};
use proptest::prelude::*;

/// Generator: pick a syntax variant for passing a multi-value option.
/// 0 = --opt val, 1 = --opt=val, 2 = -o val, 3 = -oval (attached short)
fn arb_syntax_variant() -> impl Strategy<Value = u32> {
    0u32..4
}

/// Build CLI args for a single value using the chosen syntax variant.
fn build_arg(long: &str, short: char, value: &str, variant: u32) -> Vec<String> {
    match variant {
        0 => vec![format!("--{}", long), value.to_string()],
        1 => vec![format!("--{}={}", long, value)],
        2 => vec![format!("-{}", short), value.to_string()],
        3 => vec![format!("-{}{}", short, value)],
        _ => unreachable!(),
    }
}

// Multi-value collection preserves order
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    #[test]
    fn prop1_multi_value_preserves_order(
        values in prop::collection::vec(arb_value_string(), 1..=5),
        variants in prop::collection::vec(arb_syntax_variant(), 5),
    ) {
        let long = "items";
        let short = 'i';

        let parser = ArgBuilder::new()
            .option(Opt::new(long).placeholder("ITEM").desc("items to collect").short(short).multi())
            .build()
            .unwrap();

        let mut args: Vec<String> = Vec::new();
        for (i, val) in values.iter().enumerate() {
            args.extend(build_arg(long, short, val, variants[i % variants.len()]));
        }

        let result = parser.parse(args).unwrap();
        let collected = result.get_option_values(long);

        prop_assert_eq!(collected.len(), values.len(),
            "Expected {} values, got {}", values.len(), collected.len());
        for (i, (got, expected)) in collected.iter().zip(values.iter()).enumerate() {
            prop_assert_eq!(got, expected,
                "Mismatch at index {}: got '{}', expected '{}'", i, got, expected);
        }
    }
}

// Absent multi-value option yields empty vector
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    #[test]
    fn prop2_absent_multi_value_yields_empty(
        long in arb_safe_identifier(),
    ) {
        let parser = ArgBuilder::new()
            .option(Opt::new(&long).placeholder("VAL").desc("a multi option").multi())
            .build()
            .unwrap();

        let result = parser.parse(vec![]).unwrap();
        let collected = result.get_option_values(&long);

        prop_assert!(collected.is_empty(),
            "Expected empty vector for absent multi-value option, got {:?}", collected);
    }
}

// get_option returns last value for multi-value options
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    #[test]
    fn prop3_get_option_returns_last_value(
        values in prop::collection::vec(arb_value_string(), 1..=5),
    ) {
        let long = "tags";

        let parser = ArgBuilder::new()
            .option(Opt::new(long).placeholder("TAG").desc("tags").multi())
            .build()
            .unwrap();

        let args: Vec<String> = values.iter()
            .flat_map(|v| vec![format!("--{}", long), v.clone()])
            .collect();

        let result = parser.parse(args).unwrap();
        let last = result.get_option(long);

        prop_assert_eq!(last, Some(values.last().unwrap().as_str()),
            "get_option should return last value");
    }
}

// get_option_values_parsed correctly parses all values
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    #[test]
    fn prop4_get_option_values_parsed(
        numbers in prop::collection::vec(-1000i64..1000, 1..=5),
    ) {
        let long = "nums";

        let parser = ArgBuilder::new()
            .option(Opt::new(long).placeholder("NUM").desc("numbers").multi())
            .build()
            .unwrap();

        let args: Vec<String> = numbers.iter()
            .flat_map(|n| vec![format!("--{}", long), n.to_string()])
            .collect();

        let result = parser.parse(args).unwrap();
        let parsed: Vec<Result<i64, _>> = result.get_option_values_parsed(long);

        prop_assert_eq!(parsed.len(), numbers.len());
        for (i, (got, expected)) in parsed.iter().zip(numbers.iter()).enumerate() {
            match got {
                Ok(v) => prop_assert_eq!(v, expected,
                    "Mismatch at index {}: got {}, expected {}", i, v, expected),
                Err(e) => prop_assert!(false, "Parse error at index {}: {:?}", i, e),
            }
        }
    }
}

// Env var fallback splits by comma for multi-value options
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    #[test]
    fn prop5_env_var_fallback_comma_split(
        values in prop::collection::vec(arb_value_string(), 1..=5),
        env_suffix in arb_env_var_name(),
        run_id in 0u64..u64::MAX,
    ) {
        let long = "include";
        let env_var = format!("NANOARGS_MV5_{}_{}", run_id, env_suffix);

        let env_value = values.join(",");
        std::env::set_var(&env_var, &env_value);

        let parser = ArgBuilder::new()
            .option(Opt::new(long).placeholder("PAT").desc("patterns").multi().env(&env_var))
            .build()
            .unwrap();

        let result = parser.parse(vec![]);
        std::env::remove_var(&env_var);

        let r = result.unwrap();
        let collected = r.get_option_values(long);

        let expected: Vec<&str> = values.iter().map(|s| s.as_str()).filter(|s| !s.is_empty()).collect();
        let got: Vec<&str> = collected.iter().map(|s| s.as_str()).collect();

        prop_assert_eq!(&got, &expected,
            "Env var comma-split mismatch. env_value='{}'", env_value);
    }
}

// CLI values override env var for multi-value options
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    #[test]
    fn prop6_cli_overrides_env_var(
        cli_values in prop::collection::vec(arb_value_string(), 1..=3),
        env_values in prop::collection::vec(arb_value_string(), 1..=3),
        env_suffix in arb_env_var_name(),
        run_id in 0u64..u64::MAX,
    ) {
        let long = "tags";
        let env_var = format!("NANOARGS_MV6_{}_{}", run_id, env_suffix);

        std::env::set_var(&env_var, env_values.join(","));

        let parser = ArgBuilder::new()
            .option(Opt::new(long).placeholder("TAG").desc("tags").multi().env(&env_var))
            .build()
            .unwrap();

        let args: Vec<String> = cli_values.iter()
            .flat_map(|v| vec![format!("--{}", long), v.clone()])
            .collect();

        let result = parser.parse(args);
        std::env::remove_var(&env_var);

        let r = result.unwrap();
        let collected = r.get_option_values(long);

        prop_assert_eq!(collected, cli_values.as_slice(),
            "CLI values should override env var. Got {:?}, expected {:?}", collected, cli_values);
    }
}

// Help text contains "(multiple)" iff multi is true
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    #[test]
    fn prop7_help_text_multiple_indicator(
        long in arb_safe_identifier(),
        desc in arb_safe_description(),
        is_multi in any::<bool>(),
    ) {
        let mut o = Opt::new(&long).placeholder("VAL").desc(&desc);
        if is_multi { o = o.multi(); }
        let parser = ArgBuilder::new().option(o).build().unwrap();
        let help = parser.help_text();

        if is_multi {
            prop_assert!(help.contains("(multiple)"),
                "Help text should contain '(multiple)' for multi-value option.\nHelp:\n{}", help);
        } else {
            prop_assert!(!help.contains("(multiple)"),
                "Help text should NOT contain '(multiple)' for single-value option.\nHelp:\n{}", help);
        }
    }
}

// Multi-value collection works in subcommands
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    #[test]
    fn prop9_multi_value_in_subcommands(
        global_values in prop::collection::vec(arb_value_string(), 1..=3),
        sub_values in prop::collection::vec(arb_value_string(), 1..=3),
        sub_name in arb_subcommand_name(),
    ) {
        let sub_parser = ArgBuilder::new()
            .option(Opt::new("sub-items").placeholder("ITEM").desc("sub items").short('s').multi())
            .build()
            .unwrap();

        let parser = ArgBuilder::new()
            .option(Opt::new("global-items").placeholder("ITEM").desc("global items").short('g').multi())
            .subcommand(&sub_name, "a subcommand", sub_parser)
            .build()
            .unwrap();

        let mut args: Vec<String> = Vec::new();
        for v in &global_values {
            args.push("--global-items".to_string());
            args.push(v.clone());
        }
        args.push(sub_name.clone());
        for v in &sub_values {
            args.push("--sub-items".to_string());
            args.push(v.clone());
        }

        let result = parser.parse(args).unwrap();

        let global_collected = result.get_option_values("global-items");
        prop_assert_eq!(global_collected, global_values.as_slice(),
            "Global multi-value mismatch");

        let sub_result = result.subcommand_result().unwrap();
        let sub_collected = sub_result.get_option_values("sub-items");
        prop_assert_eq!(sub_collected, sub_values.as_slice(),
            "Subcommand multi-value mismatch");
    }
}

// Default value produces single-element vector
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    #[test]
    fn prop10_default_produces_single_element(
        long in arb_safe_identifier(),
        default_val in arb_value_string(),
    ) {
        let parser = ArgBuilder::new()
            .option(Opt::new(&long).placeholder("VAL").desc("a multi option").multi().default(&default_val))
            .build()
            .unwrap();

        let result = parser.parse(vec![]).unwrap();
        let collected = result.get_option_values(&long);

        prop_assert_eq!(collected, &[default_val.clone()],
            "Default should produce single-element vector. Got {:?}, expected [{}]",
            collected, default_val);
    }
}
