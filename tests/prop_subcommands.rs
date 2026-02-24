mod common;

use common::*;
use nanoargs::{ArgBuilder, Flag, Opt, Pos};
use proptest::prelude::*;

// Subcommand registration preserves definitions
proptest! {
    #[test]
    fn prop_sub1_registration_preserves_definitions(
        subcmd_names in prop::collection::hash_set(arb_subcommand_name(), 1..5),
        descs in prop::collection::vec(arb_safe_description(), 5),
        sub_flag_longs in prop::collection::vec(
            arb_identifier().prop_filter("avoid help collision", |s| s != "help"), 5
        ),
    ) {
        let names: Vec<String> = subcmd_names.into_iter().collect();
        let mut builder = ArgBuilder::new();

        for (i, name) in names.iter().enumerate() {
            let sub_parser = ArgBuilder::new()
                .flag(Flag::new(&sub_flag_longs[i]).desc("a flag"))
                .build().unwrap();
            builder = builder.subcommand(name, &descs[i], sub_parser);
        }

        let parser = builder.build().unwrap();
        let subcmds = parser.subcommands();

        prop_assert_eq!(subcmds.len(), names.len());

        for (i, name) in names.iter().enumerate() {
            let found = subcmds.iter().find(|s| s.name == *name);
            prop_assert!(found.is_some(), "Missing subcommand: {}", name);
            let found = found.unwrap();
            prop_assert_eq!(&found.description, &descs[i]);
            prop_assert_eq!(found.parser.flags().len(), 1);
            prop_assert_eq!(&found.parser.flags()[0].long, &sub_flag_longs[i]);
        }
    }
}

// Duplicate subcommand registration overwrites
proptest! {
    #[test]
    fn prop_sub2_duplicate_registration_overwrites(
        name in arb_subcommand_name(),
        desc1 in arb_safe_description(),
        desc2 in arb_safe_description(),
        flag1 in arb_identifier().prop_filter("avoid help collision", |s| s != "help"),
        flag2 in arb_identifier().prop_filter("avoid help collision", |s| s != "help"),
    ) {
        let parser1 = ArgBuilder::new().flag(Flag::new(&flag1).desc("first")).build().unwrap();
        let parser2 = ArgBuilder::new().flag(Flag::new(&flag2).desc("second")).build().unwrap();

        let parser = ArgBuilder::new()
            .subcommand(&name, &desc1, parser1)
            .subcommand(&name, &desc2, parser2)
            .build().unwrap();

        let subcmds = parser.subcommands();
        prop_assert_eq!(subcmds.len(), 1);
        prop_assert_eq!(&subcmds[0].name, &name);
        prop_assert_eq!(&subcmds[0].description, &desc2);
        prop_assert_eq!(subcmds[0].parser.flags().len(), 1);
        prop_assert_eq!(&subcmds[0].parser.flags()[0].long, &flag2);
    }
}

// Subcommands coexist with global definitions
proptest! {
    #[test]
    fn prop_sub3_subcommands_coexist_with_global_definitions(
        flags in prop::collection::vec(arb_safe_flag_def(), 0..3),
        options in prop::collection::vec(arb_safe_option_def(), 0..3),
        positionals in prop::collection::vec(arb_safe_positional_def(), 0..3),
        subcmd_names in prop::collection::hash_set(arb_subcommand_name(), 1..4),
        subcmd_descs in prop::collection::vec(arb_safe_description(), 4),
    ) {
        {
            let mut longs = std::collections::HashSet::new();
            let mut shorts = std::collections::HashSet::new();
            for (long, short, _) in &flags {
                prop_assume!(longs.insert(long.clone()));
                if let Some(ch) = short { prop_assume!(shorts.insert(*ch)); }
            }
            for (long, short, _, _, _, _) in &options {
                prop_assume!(longs.insert(long.clone()));
                if let Some(ch) = short { prop_assume!(shorts.insert(*ch)); }
            }
        }

        let mut builder = ArgBuilder::new();
        for (long, short, desc) in &flags {
            let mut f = Flag::new(long).desc(desc);
            if let Some(ch) = short { f = f.short(*ch); }
            builder = builder.flag(f);
        }
        for (long, short, placeholder, desc, required, default) in &options {
            let mut o = Opt::new(long).placeholder(placeholder).desc(desc);
            if let Some(ch) = short { o = o.short(*ch); }
            if *required {
                o = o.required();
            } else if let Some(def) = default {
                o = o.default(def);
            }
            builder = builder.option(o);
        }
        for (name, desc, required) in &positionals {
            let mut p = Pos::new(name).desc(desc);
            if *required { p = p.required(); }
            builder = builder.positional(p);
        }

        let names: Vec<String> = subcmd_names.into_iter().collect();
        for (i, name) in names.iter().enumerate() {
            let sub_parser = ArgBuilder::new().build().unwrap();
            builder = builder.subcommand(name, &subcmd_descs[i], sub_parser);
        }

        let parser = builder.build().unwrap();

        prop_assert_eq!(parser.flags().len(), flags.len());
        prop_assert_eq!(parser.options().len(), options.len());
        prop_assert_eq!(parser.positionals().len(), positionals.len());
        prop_assert_eq!(parser.subcommands().len(), names.len());

        for name in &names {
            prop_assert!(
                parser.subcommands().iter().any(|s| s.name == *name),
                "Missing subcommand: {}", name
            );
        }
    }
}

// Subcommand parse delegation
proptest! {
    #[test]
    fn prop_sub4_subcommand_parse_delegation(
        subcmd_name in arb_subcommand_name(),
        sub_flag_long in arb_identifier().prop_filter("avoid help collision", |s| s != "help"),
        sub_flag_present in any::<bool>(),
    ) {
        let sub_parser = ArgBuilder::new()
            .flag(Flag::new(&sub_flag_long).desc("sub flag"))
            .build().unwrap();

        let parent = ArgBuilder::new()
            .subcommand(&subcmd_name, "test subcommand", sub_parser)
            .build().unwrap();

        let mut args: Vec<String> = vec![subcmd_name.clone()];
        if sub_flag_present {
            args.push(format!("--{sub_flag_long}"));
        }

        let result = parent.parse(args).unwrap();
        prop_assert_eq!(result.subcommand(), Some(subcmd_name.as_str()));

        let sub_result = result.subcommand_result().unwrap();
        prop_assert_eq!(sub_result.get_flag(&sub_flag_long), sub_flag_present);
    }
}

// Global args parsed alongside subcommand args
proptest! {
    #[test]
    fn prop_sub5_global_args_alongside_subcommand(
        subcmd_name in arb_subcommand_name(),
        global_flag_long in arb_identifier()
            .prop_filter("avoid help collision", |s| s != "help")
            .prop_filter("prefix g", |s| s.starts_with('g')),
        sub_flag_long in arb_identifier()
            .prop_filter("avoid help collision", |s| s != "help")
            .prop_filter("prefix s", |s| s.starts_with('s')),
        global_flag_present in any::<bool>(),
        sub_flag_present in any::<bool>(),
    ) {
        prop_assume!(global_flag_long != sub_flag_long);

        let sub_parser = ArgBuilder::new()
            .flag(Flag::new(&sub_flag_long).desc("sub flag"))
            .build().unwrap();

        let parent = ArgBuilder::new()
            .flag(Flag::new(&global_flag_long).desc("global flag"))
            .subcommand(&subcmd_name, "test subcommand", sub_parser)
            .build().unwrap();

        let mut args: Vec<String> = Vec::new();
        if global_flag_present {
            args.push(format!("--{global_flag_long}"));
        }
        args.push(subcmd_name.clone());
        if sub_flag_present {
            args.push(format!("--{sub_flag_long}"));
        }

        let result = parent.parse(args).unwrap();
        prop_assert_eq!(result.get_flag(&global_flag_long), global_flag_present);
        prop_assert_eq!(result.subcommand(), Some(subcmd_name.as_str()));

        let sub_result = result.subcommand_result().unwrap();
        prop_assert_eq!(sub_result.get_flag(&sub_flag_long), sub_flag_present);
    }
}

// Help text contains all subcommand names and descriptions
proptest! {
    #[test]
    fn prop_sub6_help_text_contains_all_subcommands(
        subcmd_names in prop::collection::hash_set(arb_subcommand_name(), 1..5),
        descs in prop::collection::vec(arb_safe_description(), 5),
    ) {
        let names: Vec<String> = subcmd_names.into_iter().collect();
        let mut builder = ArgBuilder::new().name("testprog");

        for (i, name) in names.iter().enumerate() {
            let sub_parser = ArgBuilder::new().build().unwrap();
            builder = builder.subcommand(name, &descs[i], sub_parser);
        }

        let parser = builder.build().unwrap();
        let help = parser.help_text();

        prop_assert!(help.contains("<SUBCOMMAND>"));
        prop_assert!(help.contains("Subcommands:"));

        for (i, name) in names.iter().enumerate() {
            prop_assert!(help.contains(name.as_str()));
            prop_assert!(help.contains(descs[i].as_str()));
        }
    }
}

// Backward compatibility without subcommands
proptest! {
    #[test]
    fn prop_sub7_backward_compatibility_without_subcommands(
        flag_long in arb_identifier().prop_filter("avoid help collision", |s| s != "help"),
        flag_short in any::<u32>().prop_map(|v| {
            let c = (b'a' + (v % 26) as u8) as char;
            if c == 'h' { 'z' } else { c }
        }),
        opt_long in arb_identifier()
            .prop_filter("avoid help collision", |s| s != "help")
            .prop_filter("prefix o", |s| s.starts_with('o')),
        opt_value in "[a-zA-Z0-9]{1,10}",
        pos_value in "[a-zA-Z0-9]{1,10}",
        flag_present in any::<bool>(),
        opt_present in any::<bool>(),
    ) {
        prop_assume!(flag_long != opt_long);

        let parser = ArgBuilder::new()
            .flag(Flag::new(&flag_long).desc("a flag").short(flag_short))
            .option(Opt::new(&opt_long).placeholder("VAL").desc("an option"))
            .positional(Pos::new("arg").desc("a positional"))
            .build().unwrap();

        prop_assert!(parser.subcommands().is_empty());

        let mut args: Vec<String> = Vec::new();
        if flag_present {
            args.push(format!("--{flag_long}"));
        }
        if opt_present {
            args.push(format!("--{opt_long}"));
            args.push(opt_value.clone());
        }
        args.push(pos_value.clone());

        let result = parser.parse(args).unwrap();

        prop_assert_eq!(result.subcommand(), None);
        prop_assert!(result.subcommand_result().is_none());
        prop_assert_eq!(result.get_flag(&flag_long), flag_present);
        if opt_present {
            prop_assert_eq!(result.get_option(&opt_long), Some(opt_value.as_str()));
        } else {
            prop_assert_eq!(result.get_option(&opt_long), None);
        }
        prop_assert_eq!(result.get_positionals(), &[pos_value]);
    }
}
