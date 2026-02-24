mod common;

use common::*;
use nanoargs::{ArgBuilder, Opt, OptionDef};
use proptest::prelude::*;

// OptionDef equivalence across all modifier combinations
proptest! {
    #[test]
    fn prop1_option_def_equivalence(
        long in arb_identifier(),
        short in arb_short(),
        placeholder in arb_identifier(),
        desc in arb_description(),
        use_multi in any::<bool>(),
        use_required in any::<bool>(),
        default_val in prop::option::of(arb_identifier()),
        env_val in prop::option::of(arb_env_var_name()),
    ) {
        let mut o = Opt::new(&long).placeholder(&placeholder).desc(&desc);
        if let Some(ch) = short { o = o.short(ch); }
        if use_multi { o = o.multi(); }
        if use_required { o = o.required(); }
        if let Some(ref dv) = default_val { o = o.default(dv); }
        if let Some(ref ev) = env_val { o = o.env(ev); }

        let parser = ArgBuilder::new().option(o).build().unwrap();
        let opts = parser.options();
        prop_assert_eq!(opts.len(), 1);

        let expected = OptionDef {
            long: long.clone(),
            short,
            placeholder: placeholder.clone(),
            description: desc.clone(),
            required: use_required,
            default: default_val.clone(),
            env_var: env_val.clone(),
            multi: use_multi,
            hidden: false,
        };
        prop_assert_eq!(&opts[0], &expected);
    }
}

// Modifier order independence
proptest! {
    #[test]
    fn prop2_modifier_order_independence(
        long in arb_identifier(),
        short in arb_short(),
        placeholder in arb_identifier(),
        desc in arb_description(),
        default_val in arb_identifier(),
        env_val in arb_env_var_name(),
        perm in prop::sample::select(vec![
            vec![0,1,2,3], vec![0,1,3,2], vec![0,2,1,3], vec![0,2,3,1],
            vec![0,3,1,2], vec![0,3,2,1], vec![1,0,2,3], vec![1,0,3,2],
            vec![1,2,0,3], vec![1,2,3,0], vec![1,3,0,2], vec![1,3,2,0],
            vec![2,0,1,3], vec![2,0,3,1], vec![2,1,0,3], vec![2,1,3,0],
            vec![2,3,0,1], vec![2,3,1,0], vec![3,0,1,2], vec![3,0,2,1],
            vec![3,1,0,2], vec![3,1,2,0], vec![3,2,0,1], vec![3,2,1,0],
        ]),
    ) {
        // Canonical order: multi, required, default, env
        let mut canonical_opt = Opt::new(&long).placeholder(&placeholder).desc(&desc);
        if let Some(ch) = short { canonical_opt = canonical_opt.short(ch); }
        canonical_opt = canonical_opt.multi().required().default(&default_val).env(&env_val);
        let canonical = ArgBuilder::new().option(canonical_opt).build().unwrap();

        // Permuted order
        let mut o = Opt::new(&long).placeholder(&placeholder).desc(&desc);
        if let Some(ch) = short { o = o.short(ch); }
        for &step in &perm {
            o = match step {
                0 => o.multi(),
                1 => o.required(),
                2 => o.default(&default_val),
                3 => o.env(&env_val),
                _ => unreachable!(),
            };
        }
        let permuted = ArgBuilder::new().option(o).build().unwrap();

        prop_assert_eq!(&canonical.options()[0], &permuted.options()[0]);
    }
}

// Declaration order preservation
proptest! {
    #[test]
    fn prop3_declaration_order_preservation(
        count in 1usize..8,
    ) {
        let names: Vec<String> = (0..count).map(|i| format!("opt{i}")).collect();

        let mut builder = ArgBuilder::new();
        for name in &names {
            builder = builder.option(Opt::new(name).placeholder("VAL").desc("desc"));
        }
        let parser = builder.build().unwrap();
        let opts = parser.options();

        prop_assert_eq!(opts.len(), names.len());
        for (i, name) in names.iter().enumerate() {
            prop_assert_eq!(&opts[i].long, name);
        }
    }
}
