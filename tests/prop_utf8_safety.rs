mod common;

use common::strip_ansi_inline;
use nanoargs::ParseError;
use proptest::prelude::*;

// Feature: utf8-safety, Property 1: InvalidUtf8 Display contains the lossy representation
// Validates: Requirements 1.1, 1.2
proptest! {
    #[test]
    fn prop1_invalid_utf8_display_contains_lossy_representation(
        lossy in "\\PC{1,50}"
    ) {
        let err = ParseError::InvalidUtf8(lossy.clone());
        let display = strip_ansi_inline(&err.to_string());
        prop_assert!(
            display.contains(&lossy),
            "Display output {:?} did not contain lossy repr {:?}",
            display,
            lossy
        );
        prop_assert!(
            display.contains("argument is not valid UTF-8:"),
            "Display output {:?} missing expected message prefix",
            display
        );
    }
}

// Feature: utf8-safety, Property 2: Valid UTF-8 strings round-trip through OsString conversion
// Validates: Requirements 2.1, 3.1
proptest! {
    #[test]
    fn prop2_valid_utf8_roundtrips_through_osstring(
        s in "\\PC{0,100}"
    ) {
        let os = std::ffi::OsString::from(s.clone());
        let result = os.into_string();
        prop_assert!(result.is_ok(), "Valid UTF-8 string failed OsString round-trip");
        prop_assert_eq!(result.unwrap(), s);
    }
}
