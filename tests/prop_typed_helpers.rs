mod common;

use common::{arb_identifier, arb_non_numeric_string, arb_u32_string};
use nanoargs::{OptionError, ParseResultBuilder};
use proptest::prelude::*;

// Feature: or-default-error-propagation, Property 1: get_option_or_default returns Ok(parsed) on success
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
        prop_assert_eq!(actual, Ok(expected));
    }
}

// Feature: or-default-error-propagation, Property 2: get_option_or_default returns Ok(default) on absent
// **Validates: Requirements 1.2**
proptest! {
    #[test]
    fn prop_get_option_or_default_returns_default_on_absent(
        name in arb_identifier(),
        default_val in any::<u32>(),
    ) {
        // Build a ParseResult WITHOUT the option
        let result = ParseResultBuilder::new().build();
        let actual = result.get_option_or_default::<u32>(&name, default_val);
        prop_assert_eq!(actual, Ok(default_val));
    }
}

// Feature: or-default-error-propagation, Property 3: get_option_or_default returns Err(ParseFailed) on unparseable
// **Validates: Requirements 1.3**
proptest! {
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
        match actual {
            Err(OptionError::ParseFailed { option, message }) => {
                prop_assert_eq!(option, name);
                prop_assert!(!message.is_empty(), "parse failure message should be non-empty");
            }
            other => prop_assert!(false, "expected Err(ParseFailed), got {:?}", other),
        }
    }
}

// Feature: or-default-error-propagation, Property 4: get_option_or returns Ok(parsed) without calling closure
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
        prop_assert_eq!(actual, Ok(expected));
        prop_assert!(!closure_called.get(), "closure should not be called when option is present and parseable");
    }
}

// Feature: or-default-error-propagation, Property 5: get_option_or returns Ok(closure_result) on absent
// **Validates: Requirements 2.2**
proptest! {
    #[test]
    fn prop_get_option_or_returns_closure_result_on_absent(
        name in arb_identifier(),
        fallback in any::<u32>(),
    ) {
        // Build a ParseResult WITHOUT that option
        let result = ParseResultBuilder::new().build();
        let actual = result.get_option_or::<u32, _>(&name, || fallback);
        prop_assert_eq!(actual, Ok(fallback));
    }
}

// Feature: or-default-error-propagation, Property 6: get_option_or returns Err(ParseFailed) without calling closure on unparseable
// **Validates: Requirements 2.3**
proptest! {
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

// Feature: or-default-error-propagation, Property 7: get_option_values_or_default returns Ok(parsed_vec) on all-valid
// **Validates: Requirements 3.1**
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
        prop_assert_eq!(actual, Ok(values));
    }
}

// Feature: or-default-error-propagation, Property 8: get_option_values_or_default returns Ok(default) on absent
// **Validates: Requirements 3.2**
proptest! {
    #[test]
    fn prop_get_option_values_or_default_returns_default_on_absent(
        name in arb_identifier(),
        default_values in prop::collection::vec(any::<u32>(), 0..=5),
    ) {
        // Build a ParseResult WITHOUT that option
        let result = ParseResultBuilder::new().build();
        let actual = result.get_option_values_or_default::<u32>(&name, default_values.clone());
        prop_assert_eq!(actual, Ok(default_values));
    }
}

// Feature: or-default-error-propagation, Property 9: get_option_values_or_default returns Err(ParseFailed) on any parse failure
// **Validates: Requirements 3.3**
proptest! {
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
        match actual {
            Err(OptionError::ParseFailed { option, message }) => {
                prop_assert_eq!(option, name);
                prop_assert!(!message.is_empty(), "parse failure message should be non-empty");
            }
            other => prop_assert!(false, "expected Err(ParseFailed), got {:?}", other),
        }
    }
}
