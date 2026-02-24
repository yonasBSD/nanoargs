mod common;

use common::*;
use nanoargs::{ArgBuilder, Flag, FlagDef, Opt, OptionDef, Pos, PositionalDef};
use proptest::prelude::*;

// Feature: api-inversion, Property 1: From conversion preserves all fields
// **Validates: Requirements 1.4, 2.8, 3.3**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop1_from_conversion_preserves_all_flag_fields(
        long in arb_identifier(),
        short in arb_short(),
        desc in arb_description(),
        hidden in any::<bool>(),
    ) {
        let mut f = Flag::new(&long).desc(&desc);
        if let Some(ch) = short { f = f.short(ch); }
        if hidden { f = f.hidden(); }

        let def = FlagDef::from(f);

        prop_assert_eq!(&def.long, &long);
        prop_assert_eq!(def.short, short);
        prop_assert_eq!(&def.description, &desc);
        prop_assert_eq!(def.hidden, hidden);
    }

    #[test]
    fn prop1_from_conversion_preserves_all_opt_fields(
        long in arb_identifier(),
        short in arb_short(),
        placeholder in arb_identifier(),
        desc in arb_description(),
        required in any::<bool>(),
        default in prop::option::of(arb_identifier()),
        env_var in prop::option::of(arb_env_var_name()),
        multi in any::<bool>(),
        hidden in any::<bool>(),
    ) {
        let mut o = Opt::new(&long).placeholder(&placeholder).desc(&desc);
        if let Some(ch) = short { o = o.short(ch); }
        if required { o = o.required(); }
        if let Some(ref d) = default { o = o.default(d); }
        if let Some(ref e) = env_var { o = o.env(e); }
        if multi { o = o.multi(); }
        if hidden { o = o.hidden(); }

        let def = OptionDef::from(o);

        prop_assert_eq!(&def.long, &long);
        prop_assert_eq!(def.short, short);
        prop_assert_eq!(&def.placeholder, &placeholder);
        prop_assert_eq!(&def.description, &desc);
        prop_assert_eq!(def.required, required);
        prop_assert_eq!(&def.default, &default);
        prop_assert_eq!(&def.env_var, &env_var);
        prop_assert_eq!(def.multi, multi);
        prop_assert_eq!(def.hidden, hidden);
    }

    #[test]
    fn prop1_from_conversion_preserves_all_pos_fields(
        name in arb_identifier(),
        desc in arb_description(),
        required in any::<bool>(),
    ) {
        let mut p = Pos::new(&name).desc(&desc);
        if required { p = p.required(); }

        let def = PositionalDef::from(p);

        prop_assert_eq!(&def.name, &name);
        prop_assert_eq!(&def.description, &desc);
        prop_assert_eq!(def.required, required);
    }
}

// Feature: api-inversion, Property 2: ArgBuilder preserves all definitions through new API
// **Validates: Requirements 4.4**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop2_argbuilder_preserves_definitions_through_new_api(
        flags in prop::collection::vec(arb_flag_def(), 0..5),
        options in prop::collection::vec(arb_option_def(), 0..5),
        positionals in prop::collection::vec(arb_positional_def(), 0..5),
    ) {
        // Ensure unique longs and shorts so build() succeeds
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

        let parser = builder.build().unwrap();

        // Verify counts
        prop_assert_eq!(parser.flags().len(), flags.len());
        prop_assert_eq!(parser.options().len(), options.len());
        prop_assert_eq!(parser.positionals().len(), positionals.len());

        // Verify flag fields in order
        for (i, (long, short, desc)) in flags.iter().enumerate() {
            prop_assert_eq!(&parser.flags()[i].long, long);
            prop_assert_eq!(&parser.flags()[i].short, short);
            prop_assert_eq!(&parser.flags()[i].description, desc);
        }

        // Verify option fields in order
        for (i, (long, short, placeholder, desc, required, default)) in options.iter().enumerate() {
            prop_assert_eq!(&parser.options()[i].long, long);
            prop_assert_eq!(&parser.options()[i].short, short);
            prop_assert_eq!(&parser.options()[i].placeholder, placeholder);
            prop_assert_eq!(&parser.options()[i].description, desc);
            prop_assert_eq!(parser.options()[i].required, *required);
            if *required {
                // When required, default is not set via builder
            } else {
                prop_assert_eq!(&parser.options()[i].default, default);
            }
        }

        // Verify positional fields in order
        for (i, (name, desc, required)) in positionals.iter().enumerate() {
            prop_assert_eq!(&parser.positionals()[i].name, name);
            prop_assert_eq!(&parser.positionals()[i].description, desc);
            prop_assert_eq!(parser.positionals()[i].required, *required);
        }
    }
}
