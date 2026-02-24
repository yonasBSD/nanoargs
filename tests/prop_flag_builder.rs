mod common;

use common::*;
use nanoargs::{ArgBuilder, Flag, FlagDef, Opt};
use proptest::prelude::*;

// FlagBuilder produces correct FlagDef
proptest! {
    #[test]
    fn prop1_flag_builder_produces_correct_flag_def(
        long in arb_identifier(),
        short in arb_short(),
        desc in arb_description(),
        hidden in any::<bool>(),
    ) {
        let mut f = Flag::new(&long).desc(&desc);
        if let Some(ch) = short { f = f.short(ch); }
        if hidden { f = f.hidden(); }
        let parser = ArgBuilder::new().flag(f).build().unwrap();
        let flags = parser.flags();
        prop_assert_eq!(flags.len(), 1);

        let expected = FlagDef {
            long: long.clone(),
            short,
            description: desc.clone(),
            hidden,
        };
        prop_assert_eq!(&flags[0], &expected);
    }
}

// OptionBuilder hidden modifier produces correct OptionDef
proptest! {
    #[test]
    fn prop2_option_builder_hidden_produces_correct_option_def(
        long in arb_identifier(),
        short in arb_short(),
        placeholder in arb_identifier(),
        desc in arb_description(),
        hidden in any::<bool>(),
    ) {
        let mut o = Opt::new(&long).placeholder(&placeholder).desc(&desc);
        if let Some(ch) = short { o = o.short(ch); }
        if hidden { o = o.hidden(); }
        let parser = ArgBuilder::new().option(o).build().unwrap();
        let opts = parser.options();
        prop_assert_eq!(opts.len(), 1);
        prop_assert_eq!(opts[0].hidden, hidden);
        prop_assert_eq!(&opts[0].long, &long);
        prop_assert_eq!(opts[0].short, short);
        prop_assert_eq!(&opts[0].placeholder, &placeholder);
        prop_assert_eq!(&opts[0].description, &desc);
    }
}

// Help text excludes hidden arguments
proptest! {
    #[test]
    fn prop3_help_text_excludes_hidden_arguments(
        visible_flag_long in arb_identifier().prop_map(|s| format!("vf{s}")),
        visible_flag_desc in arb_description(),
        hidden_flag_long in arb_identifier().prop_map(|s| format!("hf{s}")),
        hidden_flag_desc in arb_description(),
        visible_opt_long in arb_identifier().prop_map(|s| format!("vo{s}")),
        visible_opt_desc in arb_description(),
        hidden_opt_long in arb_identifier().prop_map(|s| format!("ho{s}")),
        hidden_opt_desc in arb_description(),
    ) {
        let parser = ArgBuilder::new()
            .flag(Flag::new(&visible_flag_long).desc(&visible_flag_desc))
            .flag(Flag::new(&hidden_flag_long).desc(&hidden_flag_desc).hidden())
            .option(Opt::new(&visible_opt_long).placeholder("VAL").desc(&visible_opt_desc))
            .option(Opt::new(&hidden_opt_long).placeholder("VAL").desc(&hidden_opt_desc).hidden())
            .build()
            .unwrap();

        let help = parser.help_text();

        prop_assert!(help.contains(&format!("--{visible_flag_long}")),
            "visible flag --{} missing from help", visible_flag_long);
        prop_assert!(help.contains(&format!("--{visible_opt_long}")),
            "visible option --{} missing from help", visible_opt_long);

        prop_assert!(!help.contains(&format!("--{hidden_flag_long}")),
            "hidden flag --{} should not appear in help", hidden_flag_long);
        prop_assert!(!help.contains(&format!("--{hidden_opt_long}")),
            "hidden option --{} should not appear in help", hidden_opt_long);

        prop_assert!(help.contains("[OPTIONS]"),
            "usage line should contain [OPTIONS] when visible args exist");
    }

    #[test]
    fn prop3_all_hidden_omits_options_section(
        flag_long in arb_identifier(),
        flag_desc in arb_description(),
        opt_long in arb_identifier().prop_filter("distinct from flag", |s| s.len() > 1),
    ) {
        let opt_long = format!("opt{opt_long}");
        let parser = ArgBuilder::new()
            .flag(Flag::new(&flag_long).desc(&flag_desc).hidden())
            .option(Opt::new(&opt_long).placeholder("VAL").desc("desc").hidden())
            .build()
            .unwrap();

        let help = parser.help_text();

        prop_assert!(!help.contains("[OPTIONS]"),
            "usage line should not contain [OPTIONS] when all args are hidden");
        prop_assert!(!help.contains("Options:"),
            "Options section should not appear when all args are hidden");
    }
}

// Hidden arguments parse normally
proptest! {
    #[test]
    fn prop4_hidden_arguments_parse_normally(
        flag_long in arb_identifier().prop_map(|s| format!("hf{s}")),
        opt_long in arb_identifier().prop_map(|s| format!("ho{s}")),
        opt_value in arb_value_string(),
    ) {
        let parser = ArgBuilder::new()
            .flag(Flag::new(&flag_long).desc("hidden flag").hidden())
            .option(Opt::new(&opt_long).placeholder("VAL").desc("hidden opt").hidden())
            .build()
            .unwrap();

        let args = vec![
            format!("--{flag_long}"),
            format!("--{opt_long}"),
            opt_value.clone(),
        ];
        let result = parser.parse(args).unwrap();

        prop_assert!(result.get_flag(&flag_long),
            "hidden flag --{} should be true after parsing", flag_long);
        prop_assert_eq!(result.get_option(&opt_long), Some(opt_value.as_str()),
            "hidden option --{} should have value {}", opt_long, opt_value);
    }
}

// Flag declaration order preserved
proptest! {
    #[test]
    fn prop6_flag_declaration_order_preserved(
        flags in prop::collection::vec(
            (arb_identifier(), arb_short(), arb_description()),
            1..=6,
        ).prop_filter("distinct longs", |fs| {
            let mut seen = std::collections::HashSet::new();
            fs.iter().all(|(l, _, _)| seen.insert(l.clone()))
        }).prop_filter("distinct shorts", |fs| {
            let mut seen = std::collections::HashSet::new();
            fs.iter().all(|(_, s, _)| {
                match s {
                    None => true,
                    Some(c) => seen.insert(*c),
                }
            })
        }).prop_filter("no h short", |fs| {
            fs.iter().all(|(_, s, _)| *s != Some('h'))
        })
    ) {
        let mut builder = ArgBuilder::new();
        for (long, short, desc) in &flags {
            let mut f = Flag::new(long).desc(desc);
            if let Some(ch) = short { f = f.short(*ch); }
            builder = builder.flag(f);
        }
        let parser = builder.build().unwrap();
        let result_flags = parser.flags();

        prop_assert_eq!(result_flags.len(), flags.len());
        for (i, (long, short, desc)) in flags.iter().enumerate() {
            prop_assert_eq!(&result_flags[i].long, long,
                "flag at index {} has wrong long name", i);
            prop_assert_eq!(result_flags[i].short, *short,
                "flag at index {} has wrong short", i);
            prop_assert_eq!(&result_flags[i].description, desc,
                "flag at index {} has wrong description", i);
        }
    }
}

// OptionBuilder modifier order independence including hidden
proptest! {
    #[test]
    fn prop7_option_builder_modifier_order_independence(
        long in arb_identifier(),
        short in arb_short(),
        placeholder in arb_identifier(),
        desc in arb_description(),
        is_required in any::<bool>(),
        default_val in prop_oneof![Just(None), arb_identifier().prop_map(Some)],
        env_var in prop_oneof![Just(None), "[A-Z][A-Z0-9_]{0,6}".prop_map(Some)],
        is_multi in any::<bool>(),
        is_hidden in any::<bool>(),
        order_seed in 0u32..120,
    ) {
        let build_option = |order: &[u8]| -> nanoargs::OptionDef {
            let mut o = Opt::new(&long).placeholder(&placeholder).desc(&desc);
            if let Some(ch) = short { o = o.short(ch); }
            for &modifier in order {
                match modifier {
                    0 if is_required => { o = o.required(); }
                    1 if default_val.is_some() => { o = o.default(default_val.as_ref().unwrap()); }
                    2 if env_var.is_some() => { o = o.env(env_var.as_ref().unwrap()); }
                    3 if is_multi => { o = o.multi(); }
                    4 if is_hidden => { o = o.hidden(); }
                    _ => {}
                }
            }
            let parser = ArgBuilder::new().option(o).build().unwrap();
            parser.options()[0].clone()
        };

        let canonical = build_option(&[0, 1, 2, 3, 4]);
        let reversed = build_option(&[4, 3, 2, 1, 0]);

        let mut perm = vec![0u8, 1, 2, 3, 4];
        let seed = order_seed as usize;
        for i in 0..5 {
            let j = (seed + i * 3) % 5;
            perm.swap(i, j);
        }
        let permuted = build_option(&perm);

        prop_assert_eq!(&canonical, &reversed,
            "canonical vs reversed order produced different OptionDefs");
        prop_assert_eq!(&canonical, &permuted,
            "canonical vs permuted order produced different OptionDefs");
    }
}
