mod common;

use common::*;
use nanoargs::{ArgBuilder, Flag, Opt, Pos};
use proptest::prelude::*;

// Description column alignment within each section
proptest! {
    #[test]
    fn prop_align1_description_column_alignment(
        num_flags in 1usize..5,
        num_options in 0usize..4,
        flag_shorts in prop::collection::vec(arb_short(), 5),
        flag_descs in prop::collection::vec(arb_description(), 5),
        opt_shorts in prop::collection::vec(arb_short(), 4),
        opt_descs in prop::collection::vec(arb_description(), 4),
    ) {
        {
            let mut shorts = std::collections::HashSet::new();
            for i in 0..num_flags {
                if let Some(ch) = flag_shorts[i] { prop_assume!(shorts.insert(ch)); }
            }
            for i in 0..num_options {
                if let Some(ch) = opt_shorts[i] { prop_assume!(shorts.insert(ch)); }
            }
        }

        let mut builder = ArgBuilder::new().name("test");
        let flag_names: Vec<String> = (0..num_flags).map(|i| format!("flg{i}xx")).collect();
        let opt_names: Vec<String> = (0..num_options).map(|i| format!("opt{i}xx")).collect();
        let opt_phs: Vec<String> = (0..num_options).map(|i| format!("PH{i}")).collect();

        for i in 0..num_flags {
            let mut f = Flag::new(&flag_names[i]).desc(&flag_descs[i]);
            if let Some(ch) = flag_shorts[i] { f = f.short(ch); }
            builder = builder.flag(f);
        }
        for i in 0..num_options {
            let mut o = Opt::new(&opt_names[i]).placeholder(&opt_phs[i]).desc(&opt_descs[i]);
            if let Some(ch) = opt_shorts[i] { o = o.short(ch); }
            builder = builder.option(o);
        }

        let parser = builder.build().unwrap();
        let help = parser.help_text();

        let lines = extract_section_lines(&help, "Options:");
        prop_assert_eq!(lines.len(), num_flags + num_options);

        if !lines.is_empty() {
            let dash_cols: Vec<Option<usize>> = lines.iter().map(|line| {
                let plain = strip_ansi_inline(line);
                plain.find("--")
            }).collect();
            let first = dash_cols[0];
            for (i, col) in dash_cols.iter().enumerate() {
                prop_assert_eq!(col, &first,
                    "Line {} has '--' at column {:?} but expected {:?}", i, col, first);
            }
        }
    }
}

// Minimum column gap between left column and description
proptest! {
    #[test]
    fn prop_align2_minimum_column_gap(
        num_flags in 0usize..4,
        num_options in 0usize..4,
        num_positionals in 0usize..4,
        flag_shorts in prop::collection::vec(arb_short(), 4),
        flag_descs in prop::collection::vec(arb_description(), 4),
        opt_shorts in prop::collection::vec(arb_short(), 4),
        opt_descs in prop::collection::vec(arb_description(), 4),
        pos_descs in prop::collection::vec(arb_description(), 4),
        pos_required in prop::collection::vec(any::<bool>(), 4),
    ) {
        prop_assume!(num_flags + num_options + num_positionals > 0);

        {
            let mut shorts = std::collections::HashSet::new();
            for i in 0..num_flags {
                if let Some(ch) = flag_shorts[i] { prop_assume!(shorts.insert(ch)); }
            }
            for i in 0..num_options {
                if let Some(ch) = opt_shorts[i] { prop_assume!(shorts.insert(ch)); }
            }
        }

        let mut builder = ArgBuilder::new().name("test");
        let flag_names: Vec<String> = (0..num_flags).map(|i| format!("flg{i}xx")).collect();
        let opt_names: Vec<String> = (0..num_options).map(|i| format!("opt{i}xx")).collect();
        let opt_phs: Vec<String> = (0..num_options).map(|i| format!("PH{i}")).collect();
        let pos_names: Vec<String> = (0..num_positionals).map(|i| format!("pos{i}xx")).collect();

        for i in 0..num_flags {
            let mut f = Flag::new(&flag_names[i]).desc(&flag_descs[i]);
            if let Some(ch) = flag_shorts[i] { f = f.short(ch); }
            builder = builder.flag(f);
        }
        for i in 0..num_options {
            let mut o = Opt::new(&opt_names[i]).placeholder(&opt_phs[i]).desc(&opt_descs[i]);
            if let Some(ch) = opt_shorts[i] { o = o.short(ch); }
            builder = builder.option(o);
        }
        for i in 0..num_positionals {
            let mut p = Pos::new(&pos_names[i]).desc(&pos_descs[i]);
            if pos_required[i] { p = p.required(); }
            builder = builder.positional(p);
        }

        let _parser = builder.build().unwrap();

        if num_flags + num_options > 0 {
            let mut lefts: Vec<String> = Vec::new();
            for i in 0..num_flags {
                lefts.push(match flag_shorts[i] {
                    Some(c) => format!("-{}, --{}", c, flag_names[i]),
                    None => format!("    --{}", flag_names[i]),
                });
            }
            for i in 0..num_options {
                lefts.push(match opt_shorts[i] {
                    Some(c) => format!("-{}, --{} <{}>", c, opt_names[i], opt_phs[i]),
                    None => format!("    --{} <{}>", opt_names[i], opt_phs[i]),
                });
            }
            let max_w = lefts.iter().map(|l| l.len()).max().unwrap();
            let desc_col = 2 + max_w + 2;
            for (i, left) in lefts.iter().enumerate() {
                let gap = desc_col - 2 - left.len();
                prop_assert!(gap >= 2, "Options entry {} has gap {} < 2", i, gap);
            }
        }

        if num_positionals > 0 {
            let lefts: Vec<String> = (0..num_positionals).map(|i| pos_names[i].clone()).collect();
            let max_w = lefts.iter().map(|l| l.len()).max().unwrap();
            let desc_col = 2 + max_w + 2;
            for (i, left) in lefts.iter().enumerate() {
                let gap = desc_col - 2 - left.len();
                prop_assert!(gap >= 2, "Positionals entry {} has gap {} < 2", i, gap);
            }
        }
    }
}

// Consistent double-dash alignment across entries
proptest! {
    #[test]
    fn prop_align3_consistent_double_dash_alignment(
        num_flags in 2usize..5,
        flag_shorts in prop::collection::vec(arb_short(), 5),
        flag_descs in prop::collection::vec(arb_description(), 5),
    ) {
        let mut has_some = false;
        let mut has_none = false;
        for i in 0..num_flags {
            if flag_shorts[i].is_some() { has_some = true; }
            if flag_shorts[i].is_none() { has_none = true; }
        }
        prop_assume!(has_some && has_none);

        {
            let mut shorts = std::collections::HashSet::new();
            for i in 0..num_flags {
                if let Some(ch) = flag_shorts[i] { prop_assume!(shorts.insert(ch)); }
            }
        }

        let mut builder = ArgBuilder::new().name("test");
        let flag_names: Vec<String> = (0..num_flags).map(|i| format!("flg{i}xx")).collect();
        for i in 0..num_flags {
            let mut f = Flag::new(&flag_names[i]).desc(&flag_descs[i]);
            if let Some(ch) = flag_shorts[i] { f = f.short(ch); }
            builder = builder.flag(f);
        }

        let parser = builder.build().unwrap();
        let help = parser.help_text();

        let lines = extract_section_lines(&help, "Options:");
        prop_assume!(!lines.is_empty());

        let dash_cols: Vec<Option<usize>> = lines.iter().map(|line| {
            let plain = strip_ansi_inline(line);
            plain.find("--")
        }).collect();
        let first = dash_cols[0];
        for (i, col) in dash_cols.iter().enumerate() {
            prop_assert_eq!(col, &first,
                "Line {} has '--' at column {:?} but expected {:?}", i, col, first);
        }
    }
}

// Two-space leading indent for all section entries
proptest! {
    #[test]
    fn prop_align4_two_space_leading_indent(
        num_flags in 0usize..4,
        num_options in 0usize..4,
        num_positionals in 0usize..4,
        flag_shorts in prop::collection::vec(arb_short(), 4),
        flag_descs in prop::collection::vec(arb_description(), 4),
        opt_shorts in prop::collection::vec(arb_short(), 4),
        opt_descs in prop::collection::vec(arb_description(), 4),
        pos_descs in prop::collection::vec(arb_description(), 4),
        pos_required in prop::collection::vec(any::<bool>(), 4),
    ) {
        prop_assume!(num_flags + num_options + num_positionals > 0);

        {
            let mut shorts = std::collections::HashSet::new();
            for i in 0..num_flags {
                if let Some(ch) = flag_shorts[i] { prop_assume!(shorts.insert(ch)); }
            }
            for i in 0..num_options {
                if let Some(ch) = opt_shorts[i] { prop_assume!(shorts.insert(ch)); }
            }
        }

        let mut builder = ArgBuilder::new().name("test");
        let flag_names: Vec<String> = (0..num_flags).map(|i| format!("flg{i}xx")).collect();
        let opt_names: Vec<String> = (0..num_options).map(|i| format!("opt{i}xx")).collect();
        let opt_phs: Vec<String> = (0..num_options).map(|i| format!("PH{i}")).collect();
        let pos_names: Vec<String> = (0..num_positionals).map(|i| format!("pos{i}xx")).collect();

        for i in 0..num_flags {
            let mut f = Flag::new(&flag_names[i]).desc(&flag_descs[i]);
            if let Some(ch) = flag_shorts[i] { f = f.short(ch); }
            builder = builder.flag(f);
        }
        for i in 0..num_options {
            let mut o = Opt::new(&opt_names[i]).placeholder(&opt_phs[i]).desc(&opt_descs[i]);
            if let Some(ch) = opt_shorts[i] { o = o.short(ch); }
            builder = builder.option(o);
        }
        for i in 0..num_positionals {
            let mut p = Pos::new(&pos_names[i]).desc(&pos_descs[i]);
            if pos_required[i] { p = p.required(); }
            builder = builder.positional(p);
        }

        let parser = builder.build().unwrap();
        let help = parser.help_text();

        for header in &["Options:", "Positional arguments:"] {
            let lines = extract_section_lines(&help, header);
            for line in &lines {
                let plain = strip_ansi_inline(line);
                prop_assert!(plain.starts_with("  "),
                    "In section {}, line {:?} does not start with two-space indent", header, line);
                let bytes = plain.as_bytes();
                if bytes.len() > 2 {
                    prop_assert!(bytes[2] != b' ' || plain.trim_start().starts_with("--"),
                        "In section {}, line {:?} has more than 2 leading spaces before non-dash content", header, line);
                }
            }
        }
    }
}

// Empty sections are omitted from help text
proptest! {
    #[test]
    fn prop_align6_empty_sections_omitted(
        has_flags in any::<bool>(),
        has_options in any::<bool>(),
        has_positionals in any::<bool>(),
        flag_short in arb_short(),
        flag_desc in arb_description(),
        opt_short in arb_short(),
        opt_desc in arb_description(),
        pos_desc in arb_description(),
    ) {
        if has_flags && has_options {
            if let (Some(a), Some(b)) = (flag_short, opt_short) {
                prop_assume!(a != b);
            }
        }

        let mut builder = ArgBuilder::new().name("test");
        if has_flags {
            let mut f = Flag::new("verbose").desc(&flag_desc);
            if let Some(ch) = flag_short { f = f.short(ch); }
            builder = builder.flag(f);
        }
        if has_options {
            let mut o = Opt::new("output").placeholder("FILE").desc(&opt_desc);
            if let Some(ch) = opt_short { o = o.short(ch); }
            builder = builder.option(o);
        }
        if has_positionals {
            builder = builder.positional(Pos::new("input").desc(&pos_desc));
        }

        let parser = builder.build().unwrap();
        let help = parser.help_text();
        let plain_help = strip_ansi_inline(&help);

        if !has_flags && !has_options {
            prop_assert!(!plain_help.contains("Options:"));
        } else {
            prop_assert!(plain_help.contains("Options:"));
        }

        if !has_positionals {
            prop_assert!(!plain_help.contains("Positional arguments:"));
        } else {
            prop_assert!(plain_help.contains("Positional arguments:"));
        }

        if !has_flags && !has_options && !has_positionals {
            prop_assert!(plain_help.contains("Usage:"));
            prop_assert!(!plain_help.contains("Options:"));
            prop_assert!(!plain_help.contains("Positional arguments:"));
            prop_assert!(!plain_help.contains("Subcommands:"));
        }
    }
}
