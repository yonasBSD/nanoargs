// Feature: convenience-validators
// Property tests for convenience validators

mod common;

use nanoargs::{max_length, min_length, non_empty, path_exists};
use proptest::prelude::*;

// Feature: convenience-validators, Property 1: NonEmpty validator correctness
// **Validates: Requirements 1.2, 1.3**
proptest! {
    #[test]
    fn prop_non_empty_correctness(value in ".*") {
        let v = non_empty();
        let result = v.validate(&value);
        if value.is_empty() {
            prop_assert!(result.is_err(), "Expected Err for empty string");
        } else {
            prop_assert!(result.is_ok(), "Expected Ok for non-empty string {:?}", value);
        }
    }
}

// Feature: convenience-validators, Property 2: MinLength validator correctness
// **Validates: Requirements 2.2, 2.3**
proptest! {
    #[test]
    fn prop_min_length_correctness(n in 0usize..64, value in ".{0,80}") {
        let v = min_length(n);
        let result = v.validate(&value);
        let char_count = value.chars().count();
        if char_count >= n {
            prop_assert!(result.is_ok(), "Expected Ok for char_count {} >= {}", char_count, n);
        } else {
            prop_assert!(result.is_err(), "Expected Err for char_count {} < {}", char_count, n);
        }
    }
}

// Feature: convenience-validators, Property 3: MaxLength validator correctness
// **Validates: Requirements 3.2, 3.3**
proptest! {
    #[test]
    fn prop_max_length_correctness(n in 0usize..64, value in ".{0,80}") {
        let v = max_length(n);
        let result = v.validate(&value);
        let char_count = value.chars().count();
        if char_count <= n {
            prop_assert!(result.is_ok(), "Expected Ok for char_count {} <= {}", char_count, n);
        } else {
            prop_assert!(result.is_err(), "Expected Err for char_count {} > {}", char_count, n);
        }
    }
}

// Feature: convenience-validators, Property 4: MinLength hint format
// **Validates: Requirements 2.4**
proptest! {
    #[test]
    fn prop_min_length_hint_format(n in any::<usize>()) {
        let v = min_length(n);
        let expected = format!("[min_length: {}]", n);
        prop_assert_eq!(v.hint(), Some(expected.as_str()));
    }
}

// Feature: convenience-validators, Property 5: MaxLength hint format
// **Validates: Requirements 3.4**
proptest! {
    #[test]
    fn prop_max_length_hint_format(n in any::<usize>()) {
        let v = max_length(n);
        let expected = format!("[max_length: {}]", n);
        prop_assert_eq!(v.hint(), Some(expected.as_str()));
    }
}

// Feature: convenience-validators, Property 6: PathExists rejects non-existent paths
// **Validates: Requirements 4.3**
proptest! {
    #[test]
    fn prop_path_exists_rejects_nonexistent(path in "/nonexistent_[a-z0-9]{8}/[a-z0-9]{8}") {
        let v = path_exists();
        let result = v.validate(&path);
        prop_assert!(result.is_err(), "Expected Err for non-existent path {:?}", path);
    }
}
