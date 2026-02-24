mod common;

use common::{arb_identifier, arb_non_numeric_string, arb_u32_string};
use nanoargs::{OptionError, ParseResultBuilder};
use proptest::prelude::*;

// Feature: typed-option-helpers, Property 1: get_option_or_default returns parsed value on success
// **Validates: Requirements 1.1**
proptest! {
    #[test]
    fn prop_get_option_or_default_returns_parsed_value_on_success(
        name in arb_identifier(),
        val_str in arb_u32_string(),
        default_val in any::<u32>(),
    ) {
        let expected: u32 = val_str.parse().unwrap();
        let result = ParseResultBuilder::new()
            .option(&name, &val_str)
            .build();
        let actual = result.get_option_or_default::<u32>(&name, default_val);
        prop_assert_eq!(actual, expected);
    }
}

// Feature: typed-option-helpers, Property 2: get_option_or_default returns default on absent or unparseable
// **Validates: Requirements 1.2, 1.3**
proptest! {
    #[test]
    fn prop_get_option_or_default_returns_default_on_absent(
        name in arb_identifier(),
        default_val in any::<u32>(),
    ) {
        // Build a ParseResult WITHOUT the option
        let result = ParseResultBuilder::new().build();
        let actual = result.get_option_or_default::<u32>(&name, default_val);
        prop_assert_eq!(actual, default_val);
    }

    #[test]
    fn prop_get_option_or_default_returns_default_on_unparseable(
        name in arb_identifier(),
        bad_val in arb_non_numeric_string(),
        default_val in any::<u32>(),
    ) {
        // Build a ParseResult WITH the option set to a non-numeric string
        let result = ParseResultBuilder::new()
            .option(&name, &bad_val)
            .build();
        let actual = result.get_option_or_default::<u32>(&name, default_val);
        prop_assert_eq!(actual, default_val);
    }
}

// Feature: typed-option-helpers, Property 3: get_option_or returns parsed value without calling closure
// **Validates: Requirements 2.1**
proptest! {
    #[test]
    fn prop_get_option_or_returns_parsed_value_without_calling_closure(
        name in arb_identifier(),
        val_str in arb_u32_string(),
    ) {
        let expected: u32 = val_str.parse().unwrap();
        let result = ParseResultBuilder::new()
            .option(&name, &val_str)
            .build();
        let closure_called = std::cell::Cell::new(false);
        let actual = result.get_option_or::<u32, _>(&name, || {
            closure_called.set(true);
            0
        });
        prop_assert_eq!(actual, expected);
        prop_assert!(!closure_called.get(), "closure should not be called when option is present and parseable");
    }
}


// Feature: typed-option-helpers, Property 4: get_option_or returns closure result on absent or unparseable
// **Validates: Requirements 2.2, 2.3**
proptest! {
    #[test]
    fn prop_get_option_or_returns_closure_result_on_absent(
        name in arb_identifier(),
        fallback in any::<u32>(),
    ) {
        // Build a ParseResult WITHOUT that option
        let result = ParseResultBuilder::new().build();
        let actual = result.get_option_or::<u32, _>(&name, || fallback);
        prop_assert_eq!(actual, fallback);
    }

    #[test]
    fn prop_get_option_or_returns_closure_result_on_unparseable(
        name in arb_identifier(),
        bad_val in arb_non_numeric_string(),
        fallback in any::<u32>(),
    ) {
        // Build a ParseResult WITH that option set to a non-numeric string
        let result = ParseResultBuilder::new()
            .option(&name, &bad_val)
            .build();
        let actual = result.get_option_or::<u32, _>(&name, || fallback);
        prop_assert_eq!(actual, fallback);
    }
}

// Feature: typed-option-helpers, Property 5: get_option_required returns Ok on success
// **Validates: Requirements 3.1**
proptest! {
    #[test]
    fn prop_get_option_required_returns_ok_on_success(
        name in arb_identifier(),
        val_str in arb_u32_string(),
    ) {
        let expected: u32 = val_str.parse().unwrap();
        let result = ParseResultBuilder::new()
            .option(&name, &val_str)
            .build();
        let actual = result.get_option_required::<u32>(&name);
        prop_assert_eq!(actual, Ok(expected));
    }
}

// Feature: typed-option-helpers, Property 6: get_option_required returns Missing error on absent
// **Validates: Requirements 3.2**
proptest! {
    #[test]
    fn prop_get_option_required_returns_missing_error_on_absent(
        name in arb_identifier(),
    ) {
        // Build a ParseResult WITHOUT that option
        let result = ParseResultBuilder::new().build();
        let actual = result.get_option_required::<u32>(&name);
        prop_assert_eq!(actual, Err(OptionError::Missing { option: name }));
    }
}

// Feature: typed-option-helpers, Property 7: get_option_required returns ParseFailed error on bad parse
// **Validates: Requirements 3.3**
proptest! {
    #[test]
    fn prop_get_option_required_returns_parse_failed_error_on_bad_parse(
        name in arb_identifier(),
        bad_val in arb_non_numeric_string(),
    ) {
        // Build a ParseResult WITH that option set to a non-numeric string
        let result = ParseResultBuilder::new()
            .option(&name, &bad_val)
            .build();
        let actual = result.get_option_required::<u32>(&name);
        match actual {
            Err(OptionError::ParseFailed { option, message }) => {
                prop_assert_eq!(option, name);
                prop_assert!(!message.is_empty(), "parse failure message should be non-empty");
            }
            other => prop_assert!(false, "expected Err(ParseFailed), got {:?}", other),
        }
    }
}

// Feature: typed-option-helpers, Property 8: get_option_values_or_default returns parsed Vec on all-valid
// **Validates: Requirements 4.1**
proptest! {
    #[test]
    fn prop_get_option_values_or_default_returns_parsed_vec_on_all_valid(
        name in arb_identifier(),
        values in prop::collection::vec(any::<u32>(), 1..=5),
    ) {
        let mut builder = ParseResultBuilder::new();
        for v in &values {
            builder = builder.multi_option(&name, &v.to_string());
        }
        let result = builder.build();
        let actual = result.get_option_values_or_default::<u32>(&name, vec![]);
        prop_assert_eq!(actual, values);
    }
}

// Feature: typed-option-helpers, Property 9: get_option_values_or_default returns default on absent or any parse failure
// **Validates: Requirements 4.2, 4.3**
proptest! {
    #[test]
    fn prop_get_option_values_or_default_returns_default_on_absent(
        name in arb_identifier(),
        default_values in prop::collection::vec(any::<u32>(), 0..=5),
    ) {
        // Build a ParseResult WITHOUT that option
        let result = ParseResultBuilder::new().build();
        let actual = result.get_option_values_or_default::<u32>(&name, default_values.clone());
        prop_assert_eq!(actual, default_values);
    }

    #[test]
    fn prop_get_option_values_or_default_returns_default_on_any_parse_failure(
        name in arb_identifier(),
        default_values in prop::collection::vec(any::<u32>(), 0..=5),
        prefix_valid in prop::collection::vec(any::<u32>(), 0..=3),
        bad_val in arb_non_numeric_string(),
        suffix_valid in prop::collection::vec(any::<u32>(), 0..=3),
    ) {
        // Build a ParseResult with a mix of valid u32 strings and at least one non-numeric string
        let mut builder = ParseResultBuilder::new();
        for v in &prefix_valid {
            builder = builder.multi_option(&name, &v.to_string());
        }
        builder = builder.multi_option(&name, &bad_val);
        for v in &suffix_valid {
            builder = builder.multi_option(&name, &v.to_string());
        }
        let result = builder.build();
        let actual = result.get_option_values_or_default::<u32>(&name, default_values.clone());
        prop_assert_eq!(actual, default_values);
    }
}
