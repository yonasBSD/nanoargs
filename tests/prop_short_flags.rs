mod common;

use common::*;
use nanoargs::{ArgBuilder, Flag, Opt, ParseError};
use proptest::prelude::*;

// Combined flags all set true
proptest! {
    #[test]
    fn prop_sf1_combined_flags_all_set_true(
        flag_chars in arb_flag_shorts(2..=6),
    ) {
        let mut builder = ArgBuilder::new();
        for &ch in &flag_chars {
            builder = builder.flag(Flag::new(&format!("flag_{ch}")).desc("a flag").short(ch));
        }
        let parser = builder.build().unwrap();

        let subset = flag_chars.clone();
        let token: String = format!("-{}", subset.iter().collect::<String>());
        let result = parser.parse(vec![token]).unwrap();

        for &ch in &flag_chars {
            let expected = subset.contains(&ch);
            prop_assert_eq!(
                result.get_flag(&format!("flag_{ch}")),
                expected,
                "flag_{} should be {}", ch, expected
            );
        }
    }
}

// Attached value for short option
proptest! {
    #[test]
    fn prop_sf2_attached_value_for_short_option(
        opt_char in prop::sample::select(('a'..='z').filter(|&c| c != 'h').collect::<Vec<_>>()),
        value in arb_value_string(),
    ) {
        let parser = ArgBuilder::new()
            .option(Opt::new(&format!("opt_{opt_char}")).placeholder("VAL").desc("an option").short(opt_char))
            .build().unwrap();

        let token = format!("-{opt_char}{value}");
        let result = parser.parse(vec![token]).unwrap();

        prop_assert_eq!(
            result.get_option(&format!("opt_{opt_char}")),
            Some(value.as_str())
        );
    }
}

// Combined flags then option with value
proptest! {
    #[test]
    fn prop_sf3_combined_flags_then_option_attached(
        flag_chars in arb_flag_shorts(1..=4),
        value in arb_value_string(),
    ) {
        let excluded = flag_chars.clone();
        let available: Vec<char> = ('a'..='z')
            .filter(|c| *c != 'h' && !excluded.contains(c))
            .collect();
        prop_assume!(!available.is_empty());
        let opt_char = available[0];

        let mut builder = ArgBuilder::new();
        for &ch in &flag_chars {
            builder = builder.flag(Flag::new(&format!("flag_{ch}")).desc("a flag").short(ch));
        }
        builder = builder.option(Opt::new(&format!("opt_{opt_char}")).placeholder("VAL").desc("an option").short(opt_char));
        let parser = builder.build().unwrap();

        let token = format!(
            "-{}{}{}",
            flag_chars.iter().collect::<String>(),
            opt_char,
            value
        );
        let result = parser.parse(vec![token]).unwrap();

        for &ch in &flag_chars {
            prop_assert!(result.get_flag(&format!("flag_{ch}")), "flag_{ch} should be true");
        }
        prop_assert_eq!(
            result.get_option(&format!("opt_{opt_char}")),
            Some(value.as_str())
        );
    }

    #[test]
    fn prop_sf3b_combined_flags_then_option_next_token(
        flag_chars in arb_flag_shorts(1..=4),
        value in arb_value_string(),
    ) {
        let excluded = flag_chars.clone();
        let available: Vec<char> = ('a'..='z')
            .filter(|c| *c != 'h' && !excluded.contains(c))
            .collect();
        prop_assume!(!available.is_empty());
        let opt_char = available[0];

        let mut builder = ArgBuilder::new();
        for &ch in &flag_chars {
            builder = builder.flag(Flag::new(&format!("flag_{ch}")).desc("a flag").short(ch));
        }
        builder = builder.option(Opt::new(&format!("opt_{opt_char}")).placeholder("VAL").desc("an option").short(opt_char));
        let parser = builder.build().unwrap();

        let token = format!("-{}{}", flag_chars.iter().collect::<String>(), opt_char);
        let result = parser.parse(vec![token, value.clone()]).unwrap();

        for &ch in &flag_chars {
            prop_assert!(result.get_flag(&format!("flag_{ch}")), "flag_{ch} should be true");
        }
        prop_assert_eq!(
            result.get_option(&format!("opt_{opt_char}")),
            Some(value.as_str())
        );
    }
}

// Unknown character in cluster produces error
proptest! {
    #[test]
    fn prop_sf4_unknown_char_in_cluster_produces_error(
        flag_chars in arb_flag_shorts(1..=3),
        unknown_char in prop::sample::select(vec!['0', '1', '2', '3', '4', '5', '6', '7', '8', '9']),
        insert_pos in 0usize..4,
    ) {
        let mut builder = ArgBuilder::new();
        for &ch in &flag_chars {
            builder = builder.flag(Flag::new(&format!("flag_{ch}")).desc("a flag").short(ch));
        }
        let parser = builder.build().unwrap();

        let mut chars: Vec<char> = flag_chars.clone();
        let pos = insert_pos % (chars.len() + 1);
        chars.insert(pos, unknown_char);
        let token = format!("-{}", chars.iter().collect::<String>());

        let result = parser.parse(vec![token]);
        match result {
            Err(ParseError::UnknownArgument(_)) => {}
            other => prop_assert!(false, "Expected UnknownArgument, got {:?}", other),
        }
    }
}

// Missing value when option ends cluster
proptest! {
    #[test]
    fn prop_sf5_missing_value_when_option_ends_cluster(
        flag_chars in arb_flag_shorts(1..=4),
    ) {
        let excluded = flag_chars.clone();
        let available: Vec<char> = ('a'..='z')
            .filter(|c| *c != 'h' && !excluded.contains(c))
            .collect();
        prop_assume!(!available.is_empty());
        let opt_char = available[0];

        let mut builder = ArgBuilder::new();
        for &ch in &flag_chars {
            builder = builder.flag(Flag::new(&format!("flag_{ch}")).desc("a flag").short(ch));
        }
        builder = builder.option(Opt::new(&format!("opt_{opt_char}")).placeholder("VAL").desc("an option").short(opt_char));
        let parser = builder.build().unwrap();

        let token = format!("-{}{}", flag_chars.iter().collect::<String>(), opt_char);
        let result = parser.parse(vec![token]);

        prop_assert_eq!(
            result,
            Err(ParseError::MissingValue(format!("opt_{opt_char}")))
        );
    }
}

// Double-dash separator suppresses combined token expansion
proptest! {
    #[test]
    fn prop_sf6_double_dash_suppresses_expansion(
        flag_chars in arb_flag_shorts(2..=5),
    ) {
        let mut builder = ArgBuilder::new();
        for &ch in &flag_chars {
            builder = builder.flag(Flag::new(&format!("flag_{ch}")).desc("a flag").short(ch));
        }
        builder = builder.positional(nanoargs::Pos::new("rest").desc("positional"));
        let parser = builder.build().unwrap();

        let combined = format!("-{}", flag_chars.iter().collect::<String>());
        let result = parser.parse(vec!["--".to_string(), combined.clone()]).unwrap();

        for &ch in &flag_chars {
            prop_assert!(!result.get_flag(&format!("flag_{ch}")), "flag_{ch} should be false after --");
        }
        prop_assert_eq!(result.get_positionals(), &[combined]);
    }
}

// Feature: combined-short-eq, Property 1: Combined flags with equals-delimited option value
// **Validates: Requirements 1.1, 1.2, 1.5**
proptest! {
    #[test]
    fn prop_sf7_combined_flags_eq_option(
        flag_chars in prop::collection::vec(
            prop::sample::select(('a'..='z').filter(|&c| c != 'h').collect::<Vec<_>>()),
            0..=4
        ).prop_map(|v| {
            let mut seen = std::collections::HashSet::new();
            v.into_iter().filter(|c| seen.insert(*c)).collect::<Vec<char>>()
        }),
        value in arb_value_string(),
    ) {
        let excluded: std::collections::HashSet<char> = flag_chars.iter().copied().collect();
        let available: Vec<char> = ('a'..='z')
            .filter(|c| *c != 'h' && !excluded.contains(c))
            .collect();
        prop_assume!(!available.is_empty());
        let opt_char = available[0];

        let mut builder = ArgBuilder::new();
        for &ch in &flag_chars {
            builder = builder.flag(Flag::new(&format!("flag_{ch}")).desc("a flag").short(ch));
        }
        builder = builder.option(Opt::new(&format!("opt_{opt_char}")).placeholder("VAL").desc("an option").short(opt_char));
        let parser = builder.build().unwrap();

        let token = format!(
            "-{}{}={}",
            flag_chars.iter().collect::<String>(),
            opt_char,
            value
        );
        let result = parser.parse(vec![token]).unwrap();

        for &ch in &flag_chars {
            prop_assert!(result.get_flag(&format!("flag_{ch}")), "flag_{ch} should be true");
        }
        prop_assert_eq!(
            result.get_option(&format!("opt_{opt_char}")),
            Some(value.as_str())
        );
    }
}

// Feature: combined-short-eq, Property 2: All-flags before equals produces error
// **Validates: Requirements 1.3**
proptest! {
    #[test]
    fn prop_sf8_all_flags_eq_produces_error(
        flag_chars in arb_flag_shorts(1..=4),
        value in arb_value_string(),
    ) {
        let mut builder = ArgBuilder::new();
        for &ch in &flag_chars {
            builder = builder.flag(Flag::new(&format!("flag_{ch}")).desc("a flag").short(ch));
        }
        let parser = builder.build().unwrap();

        let token = format!("-{}={}", flag_chars.iter().collect::<String>(), value);
        let result = parser.parse(vec![token]);

        match result {
            Err(ParseError::UnknownArgument(_)) => {}
            other => prop_assert!(false, "Expected UnknownArgument, got {:?}", other),
        }
    }
}

// Feature: combined-short-eq, Property 3: Unregistered character in combined-short-eq produces error
// **Validates: Requirements 1.4**
proptest! {
    #[test]
    fn prop_sf9_unregistered_char_in_combined_eq_produces_error(
        flag_chars in arb_flag_shorts(1..=3),
        unknown_char in prop::sample::select(vec!['0', '1', '2', '3', '4', '5', '6', '7', '8', '9']),
        insert_pos in 0usize..4,
        value in arb_value_string(),
    ) {
        let excluded: std::collections::HashSet<char> = flag_chars.iter().copied().collect();
        let available: Vec<char> = ('a'..='z')
            .filter(|c| *c != 'h' && !excluded.contains(c))
            .collect();
        prop_assume!(!available.is_empty());
        let opt_char = available[0];

        let mut builder = ArgBuilder::new();
        for &ch in &flag_chars {
            builder = builder.flag(Flag::new(&format!("flag_{ch}")).desc("a flag").short(ch));
        }
        builder = builder.option(Opt::new(&format!("opt_{opt_char}")).placeholder("VAL").desc("an option").short(opt_char));
        let parser = builder.build().unwrap();

        // Build prefix with unknown char inserted among the flags
        let mut prefix: Vec<char> = flag_chars.clone();
        let pos = insert_pos % (prefix.len() + 1);
        prefix.insert(pos, unknown_char);

        let token = format!("-{}{}={}", prefix.iter().collect::<String>(), opt_char, value);
        let result = parser.parse(vec![token]);

        match result {
            Err(ParseError::UnknownArgument(_)) => {}
            other => prop_assert!(false, "Expected UnknownArgument, got {:?}", other),
        }
    }
}
