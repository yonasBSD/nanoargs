// Feature: value-validation
// Property tests for value validation

mod common;

use nanoargs::{Opt, OptionDef, Pos, PositionalDef, Validator};
use proptest::prelude::*;

use common::{arb_safe_description, arb_safe_identifier};

// Feature: value-validation, Property 1: Option builder stores validator
// **Validates: Requirements 1.1**
proptest! {
    #[test]
    fn prop_opt_builder_stores_validator(
        long in arb_safe_identifier(),
        desc in arb_safe_description(),
        use_validator in any::<bool>(),
    ) {
        let opt = Opt::new(&long).desc(&desc);
        let opt = if use_validator {
            opt.validate(Validator::new(|_| Ok(())))
        } else {
            opt
        };
        let def: OptionDef = opt.into();
        prop_assert_eq!(def.validator.is_some(), use_validator);
    }
}

// Feature: value-validation, Property 2: Positional builder stores validator
// **Validates: Requirements 2.1**
proptest! {
    #[test]
    fn prop_pos_builder_stores_validator(
        name in arb_safe_identifier(),
        desc in arb_safe_description(),
        use_validator in any::<bool>(),
    ) {
        let pos = Pos::new(&name).desc(&desc);
        let pos = if use_validator {
            pos.validate(Validator::new(|_| Ok(())))
        } else {
            pos
        };
        let def: PositionalDef = pos.into();
        prop_assert_eq!(def.validator.is_some(), use_validator);
    }
}

use common::arb_value_string;
use nanoargs::{ArgBuilder, ParseError};

// Feature: value-validation, Property 3: Option validation verdict matches parse outcome
// **Validates: Requirements 1.2, 1.3, 1.4**
proptest! {
    #[test]
    fn prop_option_validation_verdict_matches_parse_outcome(
        long in arb_safe_identifier(),
        value in arb_value_string(),
        accept in any::<bool>(),
    ) {
        let err_msg = format!("rejected: {}", value);
        let err_msg_clone = err_msg.clone();
        let value_clone = value.clone();
        let validator = Validator::new(move |v| {
            if accept || v != value_clone { Ok(()) } else { Err(err_msg_clone.clone()) }
        });

        let parser = ArgBuilder::new()
            .option(Opt::new(&long).validate(validator))
            .build()
            .unwrap();

        let args = vec![format!("--{}", long), value.clone()];
        let result = parser.parse(args);

        if accept {
            prop_assert!(result.is_ok(), "Expected Ok but got {:?}", result);
        } else {
            match result {
                Err(ParseError::ValidationFailed { name, message }) => {
                    prop_assert_eq!(name, long);
                    prop_assert_eq!(message, err_msg);
                }
                other => prop_assert!(false, "Expected ValidationFailed but got {:?}", other),
            }
        }
    }
}

// Feature: value-validation, Property 4: Positional validation verdict matches parse outcome
// **Validates: Requirements 2.2, 2.3, 2.4**
proptest! {
    #[test]
    fn prop_positional_validation_verdict_matches_parse_outcome(
        name in arb_safe_identifier(),
        value in arb_value_string(),
        accept in any::<bool>(),
    ) {
        let err_msg = format!("rejected: {}", value);
        let err_msg_clone = err_msg.clone();
        let value_clone = value.clone();
        let validator = Validator::new(move |v| {
            if accept || v != value_clone { Ok(()) } else { Err(err_msg_clone.clone()) }
        });

        let parser = ArgBuilder::new()
            .positional(Pos::new(&name).validate(validator))
            .build()
            .unwrap();

        let args = vec![value.clone()];
        let result = parser.parse(args);

        if accept {
            prop_assert!(result.is_ok(), "Expected Ok but got {:?}", result);
        } else {
            match result {
                Err(ParseError::ValidationFailed { name: n, message }) => {
                    prop_assert_eq!(n, name);
                    prop_assert_eq!(message, err_msg);
                }
                other => prop_assert!(false, "Expected ValidationFailed but got {:?}", other),
            }
        }
    }
}

// Feature: value-validation, Property 5: Multi-value option validates each value individually
// **Validates: Requirements 1.5**
proptest! {
    #[test]
    fn prop_multi_value_option_validates_each_value(
        long in arb_safe_identifier(),
        values in prop::collection::vec(arb_value_string(), 1..5),
        reject_idx in any::<prop::sample::Index>(),
    ) {
        let reject_pos = reject_idx.index(values.len());
        let bad_value = values[reject_pos].clone();

        let validator = Validator::new(move |v| {
            if v == bad_value { Err(format!("bad: {}", v)) } else { Ok(()) }
        });

        let parser = ArgBuilder::new()
            .option(Opt::new(&long).multi().validate(validator))
            .build()
            .unwrap();

        let mut args = Vec::new();
        for v in &values {
            args.push(format!("--{}", long));
            args.push(v.clone());
        }

        let result = parser.parse(args);

        // At least one value matches the rejected value, so parsing should fail
        match result {
            Err(ParseError::ValidationFailed { name, .. }) => {
                prop_assert_eq!(name, long);
            }
            other => prop_assert!(false, "Expected ValidationFailed but got {:?}", other),
        }
    }
}

// Feature: value-validation, Property 9: Default values are validated
// **Validates: Requirements 6.1, 6.3**
proptest! {
    #[test]
    fn prop_default_values_are_validated(
        long in arb_safe_identifier(),
        default_val in arb_value_string(),
    ) {
        let bad = default_val.clone();
        let validator = Validator::new(move |v| {
            if v == bad { Err(format!("bad default: {}", v)) } else { Ok(()) }
        });

        let parser = ArgBuilder::new()
            .option(Opt::new(&long).default(&default_val).validate(validator))
            .build()
            .unwrap();

        // Parse with no args — should use default and fail validation
        let result = parser.parse(vec![]);

        match result {
            Err(ParseError::ValidationFailed { name, message }) => {
                prop_assert_eq!(name, long);
                prop_assert!(message.contains(&default_val));
            }
            other => prop_assert!(false, "Expected ValidationFailed but got {:?}", other),
        }
    }
}

// Feature: value-validation, Property 8: ValidationFailed display contains name and message
// **Validates: Requirements 5.2**
proptest! {
    #[test]
    fn prop_validation_failed_display_contains_name_and_message(
        name in arb_safe_identifier(),
        message in arb_safe_description(),
    ) {
        let err = ParseError::ValidationFailed {
            name: name.clone(),
            message: message.clone(),
        };
        let display = format!("{}", err);
        prop_assert!(display.contains(&name), "Display '{}' should contain name '{}'", display, name);
        prop_assert!(display.contains(&message), "Display '{}' should contain message '{}'", display, message);
    }
}

use nanoargs::{one_of, range};

// Feature: value-validation, Property 6: Range validator correctness
// **Validates: Requirements 3.2, 3.3, 3.4**
proptest! {
    #[test]
    fn prop_range_validator_correctness(
        min in -1000i64..1000,
        max in -1000i64..1000,
        value_str in prop_oneof![
            any::<i64>().prop_map(|n| n.to_string()),
            "[a-zA-Z]{1,6}",
        ],
    ) {
        prop_assume!(min <= max);
        let v = range(min, max);
        let result = v.validate(&value_str);
        match value_str.parse::<i64>() {
            Ok(n) if n >= min && n <= max => {
                prop_assert!(result.is_ok(), "Expected Ok for {} in [{}..{}], got {:?}", n, min, max, result);
            }
            Ok(_) => {
                prop_assert!(result.is_err(), "Expected Err for out-of-range value {}", value_str);
            }
            Err(_) => {
                prop_assert!(result.is_err(), "Expected Err for non-numeric '{}'", value_str);
            }
        }
    }
}

// Feature: value-validation, Property 7: OneOf validator correctness
// **Validates: Requirements 4.2, 4.3**
proptest! {
    #[test]
    fn prop_one_of_validator_correctness(
        allowed in prop::collection::vec("[a-z]{1,6}", 1..6),
        input in "[a-z]{1,6}",
    ) {
        let refs: Vec<&str> = allowed.iter().map(|s| s.as_str()).collect();
        let v = one_of(&refs);
        let result = v.validate(&input);
        if allowed.contains(&input) {
            prop_assert!(result.is_ok(), "Expected Ok for '{}' in {:?}", input, allowed);
        } else {
            prop_assert!(result.is_err(), "Expected Err for '{}' not in {:?}", input, allowed);
        }
    }
}

// Feature: value-validation, Property 12: Range validator hint format
// **Validates: Requirements 7.3**
proptest! {
    #[test]
    fn prop_range_validator_hint_format(
        min in any::<i64>(),
        max in any::<i64>(),
    ) {
        prop_assume!(min <= max);
        let v = range(min, max);
        let expected = format!("[{}..{}]", min, max);
        prop_assert_eq!(v.hint(), Some(expected.as_str()));
    }
}

// Feature: value-validation, Property 13: OneOf validator hint format
// **Validates: Requirements 7.4**
proptest! {
    #[test]
    fn prop_one_of_validator_hint_format(
        allowed in prop::collection::vec("[a-z]{1,6}", 1..6),
    ) {
        let refs: Vec<&str> = allowed.iter().map(|s| s.as_str()).collect();
        let v = one_of(&refs);
        let expected = allowed.join("|");
        prop_assert_eq!(v.hint(), Some(expected.as_str()));
    }
}

use common::strip_ansi_inline;

// Feature: value-validation, Property 11: Help text reflects validator hint presence
// **Validates: Requirements 7.1, 7.2**
proptest! {
    #[test]
    fn prop_help_text_reflects_option_validator_hint(
        long in arb_safe_identifier(),
        desc in arb_safe_description(),
        hint in arb_safe_description(),
        use_hint in any::<bool>(),
    ) {
        let validator = if use_hint {
            Validator::with_hint(&hint, |_| Ok(()))
        } else {
            Validator::new(|_| Ok(()))
        };

        let parser = ArgBuilder::new()
            .option(Opt::new(&long).desc(&desc).validate(validator))
            .build()
            .unwrap();

        let help = parser.help_text();
        let plain = strip_ansi_inline(&help);

        if use_hint {
            prop_assert!(
                plain.contains(&hint),
                "Help text should contain hint '{}' but got:\n{}",
                hint,
                plain
            );
        } else {
            // No hint — the hint text should not appear as a bracketed annotation
            // (description itself is still present)
            prop_assert!(
                plain.contains(&desc),
                "Help text should contain description '{}' but got:\n{}",
                desc,
                plain
            );
        }
    }

    #[test]
    fn prop_help_text_reflects_positional_validator_hint(
        name in arb_safe_identifier(),
        desc in arb_safe_description(),
        hint in arb_safe_description(),
        use_hint in any::<bool>(),
    ) {
        let validator = if use_hint {
            Validator::with_hint(&hint, |_| Ok(()))
        } else {
            Validator::new(|_| Ok(()))
        };

        let parser = ArgBuilder::new()
            .positional(Pos::new(&name).desc(&desc).validate(validator))
            .build()
            .unwrap();

        let help = parser.help_text();
        let plain = strip_ansi_inline(&help);

        if use_hint {
            prop_assert!(
                plain.contains(&hint),
                "Help text should contain hint '{}' but got:\n{}",
                hint,
                plain
            );
        } else {
            prop_assert!(
                plain.contains(&desc),
                "Help text should contain description '{}' but got:\n{}",
                desc,
                plain
            );
        }
    }
}
