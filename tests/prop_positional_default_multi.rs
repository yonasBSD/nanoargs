mod common;

use common::*;
#[allow(unused_imports)]
use common::{extract_section_lines, strip_ansi_inline};
use nanoargs::{ArgBuilder, Pos};
use proptest::prelude::*;

// Feature: positional-default-multi, Property 1: Builder-to-def default round trip
//
// For any positional name and any default value string, constructing
// `Pos::new(name).default(value)` and converting to `PositionalDef` SHALL produce
// a def where `def.default == Some(value)`, and constructing without `.default()`
// SHALL produce `def.default == None`.
// **Validates: Requirements 1.1, 1.2, 1.3**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop1_builder_to_def_default_round_trip(
        name in arb_identifier(),
        default in prop::option::of(arb_value_string()),
    ) {
        let mut pos = Pos::new(&name);
        if let Some(ref d) = default {
            pos = pos.default(d);
        }

        let parser = ArgBuilder::new().positional(pos).build().unwrap();
        let positionals = parser.positionals();
        prop_assert_eq!(positionals.len(), 1);
        prop_assert_eq!(&positionals[0].default, &default);
    }
}

// Feature: positional-default-multi, Property 2: Builder-to-def multi round trip
//
// For any positional name, constructing `Pos::new(name).multi()` and converting
// to `PositionalDef` SHALL produce a def where `def.multi == true`, and
// constructing without `.multi()` SHALL produce `def.multi == false`.
// **Validates: Requirements 2.1, 2.2, 2.3**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop2_builder_to_def_multi_round_trip(
        name in arb_identifier(),
        multi in any::<bool>(),
    ) {
        let mut pos = Pos::new(&name);
        if multi {
            pos = pos.multi();
        }

        let parser = ArgBuilder::new().positional(pos).build().unwrap();
        let positionals = parser.positionals();
        prop_assert_eq!(positionals.len(), 1);
        prop_assert_eq!(positionals[0].multi, multi);
    }
}

// Feature: positional-default-multi, Property 3: Required conflicts are rejected
//
// For any positional name and any default value string, an ArgBuilder containing
// a positional that is both .required() and has either .default(value) or .multi()
// SHALL return Err(ParseError::InvalidFormat) from .build().
// **Validates: Requirements 3.1, 3.2**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop3_required_conflicts_are_rejected(
        name in arb_identifier(),
        default in arb_value_string(),
        conflict_type in prop_oneof![Just("default"), Just("multi")],
    ) {
        let pos = match &*conflict_type {
            "default" => Pos::new(&name).required().default(&default),
            "multi" => Pos::new(&name).required().multi(),
            _ => unreachable!(),
        };
        let result = ArgBuilder::new().positional(pos).build();
        prop_assert!(result.is_err());
        match result.unwrap_err() {
            nanoargs::ParseError::InvalidFormat(msg) => {
                prop_assert!(msg.contains(&name));
            }
            other => prop_assert!(false, "expected InvalidFormat, got {:?}", other),
        }
    }
}

// Feature: positional-default-multi, Property 4: Multi positional must be last
//
// For any list of two or more positional definitions where a multi positional
// is not the last entry, ArgBuilder::build() SHALL return Err(ParseError::InvalidFormat).
// **Validates: Requirements 3.3**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop4_multi_positional_must_be_last(
        names in prop::collection::vec(arb_identifier(), 2..=4)
            .prop_filter("need distinct names", |v| {
                let set: std::collections::HashSet<_> = v.iter().collect();
                set.len() == v.len()
            }),
        multi_idx in 0usize..3,
    ) {
        let multi_idx = multi_idx % (names.len() - 1); // ensure not last
        let mut builder = ArgBuilder::new();
        for (i, name) in names.iter().enumerate() {
            let pos = if i == multi_idx {
                Pos::new(name).multi()
            } else {
                Pos::new(name)
            };
            builder = builder.positional(pos);
        }
        let result = builder.build();
        prop_assert!(result.is_err());
        match result.unwrap_err() {
            nanoargs::ParseError::InvalidFormat(msg) => {
                prop_assert!(msg.contains(&names[multi_idx]));
            }
            other => prop_assert!(false, "expected InvalidFormat, got {:?}", other),
        }
    }
}

// Feature: positional-default-multi, Property 5: Parser default fallback
//
// For any parser with a non-required positional that has a default value, and
// for any argument list: if the positional is provided on the command line, the
// parsed positional value at that index equals the provided value; if the
// positional is not provided, the parsed positional value at that index equals
// the default value.
// **Validates: Requirements 4.1, 4.2**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop5_parser_default_fallback(
        name in arb_identifier(),
        default in arb_value_string(),
        provided in prop::option::of(arb_value_string()),
    ) {
        let pos = Pos::new(&name).default(&default);
        let parser = ArgBuilder::new().positional(pos).build().unwrap();

        let args: Vec<String> = match &provided {
            Some(val) => vec![val.clone()],
            None => vec![],
        };

        let result = parser.parse(args).unwrap();
        let positionals = result.get_positionals();

        prop_assert_eq!(positionals.len(), 1);
        match &provided {
            Some(val) => prop_assert_eq!(&positionals[0], val),
            None => prop_assert_eq!(&positionals[0], &default),
        }
    }
}

// Feature: positional-default-multi, Property 6: Help text default annotation
//
// For any positional with a non-empty default value, the help text output SHALL
// contain the substring `[default: {value}]` in the positional arguments section.
// **Validates: Requirements 5.1, 5.2**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop6_help_text_default_annotation(
        name in arb_identifier(),
        desc in arb_description(),
        default in arb_value_string(),
    ) {
        let pos = Pos::new(&name).desc(&desc).default(&default);
        let parser = ArgBuilder::new().positional(pos).build().unwrap();
        let help = parser.help_text();

        // Positional arguments section must contain [default: value]
        let section_lines = extract_section_lines(&help, "Positional arguments:");
        let section_plain: String = section_lines.iter().map(|l| strip_ansi_inline(l)).collect::<Vec<_>>().join("\n");
        prop_assert!(
            section_plain.contains(&format!("[default: {}]", default)),
            "expected [default: {}] in positional section, got:\n{}",
            default,
            section_plain
        );

        // When no default, annotation must not appear
        let pos_no_default = Pos::new(&name).desc(&desc);
        let parser2 = ArgBuilder::new().positional(pos_no_default).build().unwrap();
        let help2 = parser2.help_text();
        let plain2 = strip_ansi_inline(&help2);
        prop_assert!(
            !plain2.contains("[default:"),
            "unexpected default annotation in help without default:\n{}",
            plain2
        );
    }
}

// Feature: positional-default-multi, Property 7: Help text multi suffix
//
// For any positional marked as multi, the help text output SHALL contain the
// positional name followed by `...` in both the usage line and the positional
// arguments section.
// **Validates: Requirements 6.1, 6.2**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop7_help_text_multi_suffix(
        name in arb_identifier(),
        desc in arb_description(),
    ) {
        // Multi positional: usage line and section should have "..."
        let pos = Pos::new(&name).desc(&desc).multi();
        let parser = ArgBuilder::new().positional(pos).build().unwrap();
        let help = parser.help_text();
        let plain = strip_ansi_inline(&help);

        // Usage line should contain [name]...
        let usage_line = plain.lines().find(|l| l.starts_with("Usage:")).unwrap();
        prop_assert!(
            usage_line.contains(&format!("[{}]...", name)),
            "expected [{}]... in usage line, got: {}",
            name,
            usage_line
        );

        // Positional arguments section should contain name...
        let section_lines = extract_section_lines(&help, "Positional arguments:");
        let section_plain: String = section_lines.iter().map(|l| strip_ansi_inline(l)).collect::<Vec<_>>().join("\n");
        prop_assert!(
            section_plain.contains(&format!("{}...", name)),
            "expected {}... in positional section, got:\n{}",
            name,
            section_plain
        );

        // Non-multi positional: no "..." suffix
        let pos2 = Pos::new(&name).desc(&desc);
        let parser2 = ArgBuilder::new().positional(pos2).build().unwrap();
        let help2 = parser2.help_text();
        let plain2 = strip_ansi_inline(&help2);
        let usage2 = plain2.lines().find(|l| l.starts_with("Usage:")).unwrap();
        prop_assert!(
            !usage2.contains(&format!("{}...", name)),
            "unexpected ... suffix in usage for non-multi positional: {}",
            usage2
        );
    }
}
