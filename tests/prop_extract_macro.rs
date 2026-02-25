mod common;

use nanoargs::{extract, OptionError, ParseResultBuilder};
use proptest::prelude::*;

// ── Property 1: Required field extraction succeeds with valid values ────────
// For any valid parseable string value for type u32, building a ParseResult
// with that value and extracting via a required field declaration SHALL produce
// a struct whose field equals the parsed value.
// **Validates: Requirements 1.1**
proptest! {
    #[test]
    fn prop1_required_field_extraction_succeeds_with_valid_values(
        value in any::<u32>(),
    ) {
        let result = ParseResultBuilder::new()
            .option("port", &value.to_string())
            .build();

        let extracted: Result<_, OptionError> = extract!(result, {
            port: u32,
        });

        let e = extracted.unwrap();
        prop_assert_eq!(e.port, value);
    }
}

// ── Property 2: Required field extraction fails when absent ─────────────────
// For any ParseResult that does not contain a given option, extracting via a
// required field declaration SHALL return Err(OptionError::Missing) with the
// correct option name.
// **Validates: Requirements 1.2**
proptest! {
    #[test]
    fn prop2_required_field_extraction_fails_when_absent(
        _dummy in any::<u8>(),
    ) {
        let result = ParseResultBuilder::new().build();

        let extracted = extract!(result, { port: u32 });

        match extracted {
            Err(OptionError::Missing { option }) => {
                prop_assert_eq!(option, "port");
            }
            other => {
                prop_assert!(false, "Expected Err(Missing), got {:?}", other);
            }
        }
    }
}

// ── Property 5: Flag field reflects ParseResult state ───────────────────────
// For any boolean value, building a ParseResult with a flag set to that value
// and extracting via a bool field declaration SHALL produce a struct whose
// field equals that boolean.
// **Validates: Requirements 3.1, 3.2**
proptest! {
    #[test]
    fn prop5_flag_field_reflects_parse_result_state(
        flag_val in any::<bool>(),
    ) {
        let result = ParseResultBuilder::new()
            .flag("verbose", flag_val)
            .build();

        let extracted = extract!(result, {
            verbose: bool,
        });

        let e = extracted.unwrap();
        prop_assert_eq!(e.verbose, flag_val);
    }
}

// ── Property 4: Optional field extraction reflects presence ─────────────────
// For any valid parseable string value for type u32, extracting via an optional
// field declaration SHALL produce Some(parsed_value) when the option is present
// and None when the option is absent.
// **Validates: Requirements 2.1, 2.2**
proptest! {
    #[test]
    fn prop4_optional_field_extraction_reflects_presence(
        value in any::<u32>(),
        present in any::<bool>(),
    ) {
        let mut builder = ParseResultBuilder::new();
        if present {
            builder = builder.option("level", &value.to_string());
        }
        let result = builder.build();

        let extracted = extract!(result, {
            level: Option<u32>,
        });

        let e = extracted.unwrap();
        if present {
            prop_assert_eq!(e.level, Some(value));
        } else {
            prop_assert_eq!(e.level, None);
        }
    }
}

// ── Property 6: Default field extraction succeeds with valid values ─────────
// For any valid parseable string value for type u32 and any default value,
// building a ParseResult with that option present and extracting via a default
// field declaration SHALL produce a struct whose field equals the parsed value
// (not the default).
// **Validates: Requirements 4.1**
proptest! {
    #[test]
    fn prop6_default_field_extraction_succeeds_with_valid_values(
        value in any::<u32>(),
        default_val in any::<u32>(),
    ) {
        let result = ParseResultBuilder::new()
            .option("jobs", &value.to_string())
            .build();

        let extracted = extract!(result, {
            jobs: u32 = default_val,
        });

        let e = extracted.unwrap();
        prop_assert_eq!(e.jobs, value);
    }
}

// ── Property 7: Default field uses default when absent ──────────────────────
// For any default value of type u32, building a ParseResult without the option
// and extracting via a default field declaration SHALL produce a struct whose
// field equals the default value.
// **Validates: Requirements 4.2**
proptest! {
    #[test]
    fn prop7_default_field_uses_default_when_absent(
        default_val in any::<u32>(),
    ) {
        let result = ParseResultBuilder::new().build();

        let extracted = extract!(result, {
            jobs: u32 = default_val,
        });

        let e = extracted.unwrap();
        prop_assert_eq!(e.jobs, default_val);
    }
}

// ── Property 8: Multi-value field extraction preserves order and completeness
// For any vector of valid parseable string values for type u32, building a
// ParseResult with those multi-values and extracting via a Vec<u32> field
// declaration SHALL produce a struct whose field equals the parsed vector in
// the same order. When no values are present, the field SHALL be an empty vector.
// **Validates: Requirements 5.1, 5.2**
proptest! {
    #[test]
    fn prop8_multi_value_field_extraction_preserves_order_and_completeness(
        values in prop::collection::vec(any::<u32>(), 0..20),
    ) {
        let mut builder = ParseResultBuilder::new();
        for v in &values {
            builder = builder.multi_option("tags", &v.to_string());
        }
        let result = builder.build();

        let extracted = extract!(result, {
            tags: Vec<u32>,
        });

        let e = extracted.unwrap();
        prop_assert_eq!(e.tags, values);
    }
}

// ── Property 3: Unparseable values produce ParseFailed across all parsing field types
// For any non-numeric string value, extracting it via a required field (T),
// optional field (Option<T>), default field (T = expr), or multi-value field
// (Vec<T>) where T = u32 SHALL return Err(OptionError::ParseFailed) with the
// correct option name.
// **Validates: Requirements 1.3, 2.3, 4.3, 5.3**
proptest! {
    #[test]
    fn prop3_unparseable_values_produce_parse_failed_required(
        bad in "[a-zA-Z][a-zA-Z0-9]{0,10}",
    ) {
        let result = ParseResultBuilder::new()
            .option("port", &bad)
            .build();

        let extracted = extract!(result, { port: u32 });
        match extracted {
            Err(OptionError::ParseFailed { option, .. }) => {
                prop_assert_eq!(option, "port");
            }
            other => prop_assert!(false, "Expected ParseFailed, got {:?}", other),
        }
    }

    #[test]
    fn prop3_unparseable_values_produce_parse_failed_optional(
        bad in "[a-zA-Z][a-zA-Z0-9]{0,10}",
    ) {
        let result = ParseResultBuilder::new()
            .option("level", &bad)
            .build();

        let extracted = extract!(result, { level: Option<u32> });
        match extracted {
            Err(OptionError::ParseFailed { option, .. }) => {
                prop_assert_eq!(option, "level");
            }
            other => prop_assert!(false, "Expected ParseFailed, got {:?}", other),
        }
    }

    #[test]
    fn prop3_unparseable_values_produce_parse_failed_default(
        bad in "[a-zA-Z][a-zA-Z0-9]{0,10}",
    ) {
        let result = ParseResultBuilder::new()
            .option("jobs", &bad)
            .build();

        let extracted = extract!(result, { jobs: u32 = 4 });
        match extracted {
            Err(OptionError::ParseFailed { option, .. }) => {
                prop_assert_eq!(option, "jobs");
            }
            other => prop_assert!(false, "Expected ParseFailed, got {:?}", other),
        }
    }

    #[test]
    fn prop3_unparseable_values_produce_parse_failed_multi(
        bad in "[a-zA-Z][a-zA-Z0-9]{0,10}",
    ) {
        let result = ParseResultBuilder::new()
            .multi_option("tags", &bad)
            .build();

        let extracted = extract!(result, { tags: Vec<u32> });
        match extracted {
            Err(OptionError::ParseFailed { option, .. }) => {
                prop_assert_eq!(option, "tags");
            }
            other => prop_assert!(false, "Expected ParseFailed, got {:?}", other),
        }
    }
}

// ── Unit tests for custom name mapping and reference semantics ──────────
// **Validates: Requirements 6.1, 7.1, 7.2, 8.2, 10.2**

#[test]
fn test_underscore_to_hyphen_name_mapping() {
    let result = ParseResultBuilder::new().option("listen-port", "8080").build();

    let extracted = extract!(result, { listen_port: u16 });

    let e = extracted.unwrap();
    assert_eq!(e.listen_port, 8080);
}

#[test]
fn test_custom_name_override_required() {
    let result = ParseResultBuilder::new().option("p", "3000").build();

    let extracted = extract!(result, { port: u16 as "p" });

    let e = extracted.unwrap();
    assert_eq!(e.port, 3000);
}

#[test]
fn test_custom_name_with_default_present() {
    let result = ParseResultBuilder::new().option("j", "8").build();

    let extracted = extract!(result, { jobs: u32 as "j" = 4 });

    let e = extracted.unwrap();
    assert_eq!(e.jobs, 8);
}

#[test]
fn test_custom_name_with_default_absent() {
    let result = ParseResultBuilder::new().build();

    let extracted = extract!(result, { jobs: u32 as "j" = 4 });

    let e = extracted.unwrap();
    assert_eq!(e.jobs, 4);
}

#[test]
fn test_parse_result_accessible_after_extract_by_reference() {
    let result = ParseResultBuilder::new().option("port", "8080").flag("verbose", true).build();

    let extracted = extract!(&result, { port: u16 });

    let e = extracted.unwrap();
    assert_eq!(e.port, 8080);

    // ParseResult is still usable after extraction by reference
    assert!(result.get_flag("verbose"));
    assert_eq!(result.get_option("port"), Some("8080"));
}

#[test]
fn test_fail_fast_error_ordering() {
    // Both "first" and "second" are missing; the macro should fail on the first one
    let result = ParseResultBuilder::new().build();

    let extracted = extract!(result, { first: u32, second: u32 });

    match extracted {
        Err(OptionError::Missing { option }) => {
            assert_eq!(option, "first");
        }
        other => panic!("Expected Err(Missing) for 'first', got {:?}", other),
    }
}

#[test]
fn test_mixed_field_types_single_extraction() {
    let result = ParseResultBuilder::new()
        .option("host", "localhost")
        .option("port", "9090")
        .flag("verbose", true)
        .option("j", "2")
        .multi_option("tags", "a")
        .multi_option("tags", "b")
        .build();

    let extracted = extract!(result, {
        host: String,
        port: u16,
        verbose: bool,
        jobs: u32 as "j" = 4,
        tags: Vec<String>,
    });

    let e = extracted.unwrap();
    assert_eq!(e.host, "localhost");
    assert_eq!(e.port, 9090);
    assert!(e.verbose);
    assert_eq!(e.jobs, 2);
    assert_eq!(e.tags, vec!["a".to_string(), "b".to_string()]);
}

// ── Feature: positional-extract, Property 1: Required positional extraction succeeds with valid values
// For any valid parseable string value for type u32, building a ParseResult
// with that value as a positional and extracting via a required positional field
// declaration (T as @pos) SHALL produce a struct whose field equals the parsed value.
// **Validates: Requirements 1.1**
proptest! {
    #[test]
    fn prop_pos1_required_positional_extraction_succeeds(
        value in any::<u32>(),
    ) {
        let result = ParseResultBuilder::new()
            .positional(&value.to_string())
            .build();

        let extracted: Result<_, OptionError> = extract!(result, {
            input: u32 as @pos,
        });

        let e = extracted.unwrap();
        prop_assert_eq!(e.input, value);
    }
}

// ── Unit test: required positional missing returns Err(Missing) with field name
// **Validates: Requirements 1.2**
#[test]
fn test_required_positional_missing_error() {
    let result = ParseResultBuilder::new().build();

    let extracted = extract!(result, { input: u32 as @pos });

    match extracted {
        Err(OptionError::Missing { option }) => {
            assert_eq!(option, "input");
        }
        other => panic!("Expected Err(Missing), got {:?}", other),
    }
}

// ── Feature: positional-extract, Property 2: Optional positional extraction reflects presence
// For any valid parseable string value for type u32, extracting via an optional
// positional field declaration (Option<T> as @pos) SHALL produce Some(parsed_value)
// when the positional is present and None when the positional is absent.
// **Validates: Requirements 2.1, 2.2**
proptest! {
    #[test]
    fn prop_pos2_optional_positional_extraction_reflects_presence(
        value in any::<u32>(),
        present in any::<bool>(),
    ) {
        let mut builder = ParseResultBuilder::new();
        if present {
            builder = builder.positional(&value.to_string());
        }
        let result = builder.build();

        let extracted: Result<_, OptionError> = extract!(result, {
            extra: Option<u32> as @pos,
        });

        let e = extracted.unwrap();
        if present {
            prop_assert_eq!(e.extra, Some(value));
        } else {
            prop_assert_eq!(e.extra, None);
        }
    }
}

// ── Feature: positional-extract, Property 3: Default positional uses value when present, default when absent
// For any valid parseable string value for type u32 and any default value,
// extracting via a default positional field declaration (T as @pos = expr)
// SHALL produce the parsed value when the positional is present, and the
// default value when the positional is absent.
// **Validates: Requirements 4.1, 4.2**
proptest! {
    #[test]
    fn prop_pos3_default_positional_uses_value_or_default(
        value in any::<u32>(),
        default_val in any::<u32>(),
        present in any::<bool>(),
    ) {
        let mut builder = ParseResultBuilder::new();
        if present {
            builder = builder.positional(&value.to_string());
        }
        let result = builder.build();

        let extracted: Result<_, OptionError> = extract!(result, {
            mode: u32 as @pos = default_val,
        });

        let e = extracted.unwrap();
        if present {
            prop_assert_eq!(e.mode, value);
        } else {
            prop_assert_eq!(e.mode, default_val);
        }
    }
}

// ── Feature: positional-extract, Property 4: Remaining positionals collection preserves order and completeness
// For any vector of valid parseable string values for type u32, building a
// ParseResult with those values as positionals and extracting via a Vec<u32> as @pos
// field declaration SHALL produce a struct whose field equals the parsed vector
// in the same order. When no positionals are present, the field SHALL be an empty vector.
// **Validates: Requirements 5.1, 5.2**
proptest! {
    #[test]
    fn prop_pos4_remaining_positionals_preserves_order_and_completeness(
        values in prop::collection::vec(any::<u32>(), 0..20),
    ) {
        let mut builder = ParseResultBuilder::new();
        for v in &values {
            builder = builder.positional(&v.to_string());
        }
        let result = builder.build();

        let extracted: Result<_, OptionError> = extract!(result, {
            files: Vec<u32> as @pos,
        });

        let e = extracted.unwrap();
        prop_assert_eq!(e.files, values);
    }
}

// ── Feature: positional-extract, Property 5: Multiple positionals get correct indices
// For any two valid parseable string values for type u32, building a ParseResult
// with both as positionals and extracting via two required positional field
// declarations SHALL produce a struct where the first positional field equals
// the first value and the second positional field equals the second value.
// **Validates: Requirements 3.1, 3.2**
proptest! {
    #[test]
    fn prop_pos5_multiple_positionals_get_correct_indices(
        first_val in any::<u32>(),
        second_val in any::<u32>(),
    ) {
        let result = ParseResultBuilder::new()
            .positional(&first_val.to_string())
            .positional(&second_val.to_string())
            .build();

        let extracted: Result<_, OptionError> = extract!(result, {
            first: u32 as @pos,
            second: u32 as @pos,
        });

        let e = extracted.unwrap();
        prop_assert_eq!(e.first, first_val);
        prop_assert_eq!(e.second, second_val);
    }
}

// ── Feature: positional-extract, Property 6: Unparseable positional values produce ParseFailed across all positional types
// For any non-numeric string value, extracting it via a required positional (T as @pos),
// optional positional (Option<T> as @pos), default positional (T as @pos = expr),
// or remaining positionals (Vec<T> as @pos) where T = u32 SHALL return
// Err(OptionError::ParseFailed) with the correct field name.
// **Validates: Requirements 1.3, 2.3, 4.3, 5.3**
proptest! {
    #[test]
    fn prop_pos6_unparseable_required_positional(
        bad in "[a-zA-Z][a-zA-Z0-9]{0,10}",
    ) {
        let result = ParseResultBuilder::new()
            .positional(&bad)
            .build();

        let extracted = extract!(result, { input: u32 as @pos });
        match extracted {
            Err(OptionError::ParseFailed { option, .. }) => {
                prop_assert_eq!(option, "input");
            }
            other => prop_assert!(false, "Expected ParseFailed, got {:?}", other),
        }
    }

    #[test]
    fn prop_pos6_unparseable_optional_positional(
        bad in "[a-zA-Z][a-zA-Z0-9]{0,10}",
    ) {
        let result = ParseResultBuilder::new()
            .positional(&bad)
            .build();

        let extracted = extract!(result, { extra: Option<u32> as @pos });
        match extracted {
            Err(OptionError::ParseFailed { option, .. }) => {
                prop_assert_eq!(option, "extra");
            }
            other => prop_assert!(false, "Expected ParseFailed, got {:?}", other),
        }
    }

    #[test]
    fn prop_pos6_unparseable_default_positional(
        bad in "[a-zA-Z][a-zA-Z0-9]{0,10}",
    ) {
        let result = ParseResultBuilder::new()
            .positional(&bad)
            .build();

        let extracted = extract!(result, { mode: u32 as @pos = 42 });
        match extracted {
            Err(OptionError::ParseFailed { option, .. }) => {
                prop_assert_eq!(option, "mode");
            }
            other => prop_assert!(false, "Expected ParseFailed, got {:?}", other),
        }
    }

    #[test]
    fn prop_pos6_unparseable_vec_positional(
        bad in "[a-zA-Z][a-zA-Z0-9]{0,10}",
    ) {
        let result = ParseResultBuilder::new()
            .positional(&bad)
            .build();

        let extracted = extract!(result, { files: Vec<u32> as @pos });
        match extracted {
            Err(OptionError::ParseFailed { option, .. }) => {
                prop_assert_eq!(option, "files");
            }
            other => prop_assert!(false, "Expected ParseFailed, got {:?}", other),
        }
    }
}

// ── Integration: positional fields interspersed with non-positional fields ──
// **Validates: Requirements 3.3, 8.1, 8.2**

#[test]
fn test_mixed_positional_and_non_positional_fields() {
    let result = ParseResultBuilder::new()
        .flag("verbose", true)
        .option("host", "localhost")
        .positional("input.txt")
        .option("port", "9090")
        .positional("output.txt")
        .multi_option("tags", "a")
        .multi_option("tags", "b")
        .build();

    // Positional fields interspersed among flags, options, and multi-values.
    // Positional indices are assigned by declaration order of @pos fields only:
    //   src  → index 0 ("input.txt")
    //   dest → index 1 ("output.txt")
    let extracted = extract!(result, {
        verbose: bool,
        src: String as @pos,
        host: String,
        dest: String as @pos,
        port: u16,
        tags: Vec<String>,
    });

    let e = extracted.unwrap();
    assert!(e.verbose);
    assert_eq!(e.src, "input.txt");
    assert_eq!(e.host, "localhost");
    assert_eq!(e.dest, "output.txt");
    assert_eq!(e.port, 9090);
    assert_eq!(e.tags, vec!["a".to_string(), "b".to_string()]);
}

#[test]
fn test_vec_positional_followed_by_required_positional_gets_missing() {
    // Vec<T> as @pos consumes all remaining positionals.
    // A required positional declared after it always sees out-of-bounds → Missing.
    let result = ParseResultBuilder::new().positional("1").positional("2").positional("3").build();

    let extracted = extract!(result, {
        all: Vec<u32> as @pos,
        leftover: u32 as @pos,
    });

    match extracted {
        Err(OptionError::Missing { option }) => {
            assert_eq!(option, "leftover");
        }
        other => panic!(
            "Expected Err(Missing) for 'leftover' after Vec consumed all, got {:?}",
            other
        ),
    }
}
