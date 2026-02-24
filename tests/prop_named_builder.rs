mod common;

use common::*;
use nanoargs::{ArgBuilder, Flag, FlagDef, Opt, OptionDef, Pos, PositionalDef};
use proptest::prelude::*;

// **Property 1: Flag new constructor produces correct FlagDef**
// **Validates: Requirements 1.1, 1.2, 1.4**
//
// For any valid long name and optional description string, constructing
// `Flag::new(long)` optionally chained with `.desc(description)` and converting
// to `FlagDef` shall produce a `FlagDef` with the matching long name, the given
// description (or empty string if `.desc()` was not called), `short` as `None`,
// and `hidden` as `false`.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop1_flag_new_constructor_produces_correct_flag_def(
        long in arb_identifier(),
        desc in prop::option::of(arb_description()),
    ) {
        let flag = if let Some(ref d) = desc {
            Flag::new(&long).desc(d)
        } else {
            Flag::new(&long)
        };

        let parser = ArgBuilder::new().flag(flag).build().unwrap();
        let flags = parser.flags();
        prop_assert_eq!(flags.len(), 1);

        let expected = FlagDef {
            long: long.clone(),
            short: None,
            description: desc.unwrap_or_default(),
            hidden: false,
        };
        prop_assert_eq!(&flags[0], &expected);
    }
}

// **Property 2: Opt new constructor produces correct OptionDef with default placeholder**
// **Validates: Requirements 2.1, 2.2, 2.3, 2.5, 5.1, 5.2, 5.3**
//
// For any valid long name, optional placeholder string, and optional description
// string, constructing `Opt::new(long)` optionally chained with `.placeholder(ph)`
// and `.desc(description)` and converting to `OptionDef` shall produce an
// `OptionDef` with the matching long name, the given placeholder (or
// `long.to_uppercase()` if `.placeholder()` was not called), the given description
// (or empty string if `.desc()` was not called), and all other fields at defaults.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop2_opt_new_constructor_produces_correct_option_def(
        long in arb_identifier(),
        placeholder in prop::option::of(arb_identifier()),
        desc in prop::option::of(arb_description()),
    ) {
        let mut opt = Opt::new(&long);
        if let Some(ref ph) = placeholder {
            opt = opt.placeholder(ph);
        }
        if let Some(ref d) = desc {
            opt = opt.desc(d);
        }

        let parser = ArgBuilder::new().option(opt).build().unwrap();
        let opts = parser.options();
        prop_assert_eq!(opts.len(), 1);

        let expected = OptionDef {
            long: long.clone(),
            short: None,
            placeholder: placeholder.unwrap_or_else(|| long.to_uppercase()),
            description: desc.unwrap_or_default(),
            required: false,
            default: None,
            env_var: None,
            multi: false,
            hidden: false,
        };
        prop_assert_eq!(&opts[0], &expected);
    }
}

// **Property 3: Pos new constructor produces correct PositionalDef**
// **Validates: Requirements 3.1, 3.2, 3.4**
//
// For any valid positional name and optional description string, constructing
// `Pos::new(name)` optionally chained with `.desc(description)` and converting
// to `PositionalDef` shall produce a `PositionalDef` with the matching name,
// the given description (or empty string if `.desc()` was not called), and
// `required` as `false`.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop3_pos_new_constructor_produces_correct_positional_def(
        name in arb_identifier(),
        desc in prop::option::of(arb_description()),
    ) {
        let pos = if let Some(ref d) = desc {
            Pos::new(&name).desc(d)
        } else {
            Pos::new(&name)
        };

        let parser = ArgBuilder::new().positional(pos).build().unwrap();
        let positionals = parser.positionals();
        prop_assert_eq!(positionals.len(), 1);

        let expected = PositionalDef {
            name: name.clone(),
            description: desc.unwrap_or_default(),
            required: false,
        };
        prop_assert_eq!(&positionals[0], &expected);
    }
}

// **Property 4: Flag modifier order independence with desc**
// **Validates: Requirements 4.1**
//
// For any valid long name, optional short char, optional description, and hidden
// flag, calling `.short()`, `.hidden()`, and `.desc()` on `Flag::new(long)` in
// any permutation shall produce the same `FlagDef`.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop4_flag_modifier_order_independence(
        long in arb_identifier(),
        short in arb_short(),
        desc in arb_description(),
        hidden in any::<bool>(),
    ) {
        // Helper: apply modifiers in a given order specified by indices [0,1,2]
        // 0 = short, 1 = hidden, 2 = desc
        let apply = |order: [usize; 3]| -> FlagDef {
            let mut f = Flag::new(&long);
            for &i in &order {
                match i {
                    0 => { if let Some(ch) = short { f = f.short(ch); } }
                    1 => { if hidden { f = f.hidden(); } }
                    2 => { f = f.desc(&desc); }
                    _ => unreachable!(),
                }
            }
            FlagDef::from(f)
        };

        let permutations: [[usize; 3]; 6] = [
            [0, 1, 2], [0, 2, 1], [1, 0, 2],
            [1, 2, 0], [2, 0, 1], [2, 1, 0],
        ];

        let baseline = apply(permutations[0]);
        for perm in &permutations[1..] {
            prop_assert_eq!(&baseline, &apply(*perm));
        }
    }
}

// **Property 5: Opt modifier order independence with desc and placeholder**
// **Validates: Requirements 4.2**
//
// For any valid long name and set of modifier values (short, required, default,
// env, multi, hidden, placeholder, description), calling the corresponding
// modifier methods on `Opt::new(long)` in any permutation shall produce the
// same `OptionDef`.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop5_opt_modifier_order_independence(
        long in arb_identifier(),
        short in arb_short(),
        desc in arb_description(),
        placeholder in arb_identifier(),
        required in any::<bool>(),
        default in prop::option::of(arb_identifier()),
        env_var in prop::option::of(arb_env_var_name()),
        multi in any::<bool>(),
        hidden in any::<bool>(),
    ) {
        // Modifiers indexed 0..8:
        // 0=short, 1=desc, 2=placeholder, 3=required, 4=default, 5=env, 6=multi, 7=hidden
        let apply = |order: &[usize]| -> OptionDef {
            let mut o = Opt::new(&long);
            for &i in order {
                match i {
                    0 => { if let Some(ch) = short { o = o.short(ch); } }
                    1 => { o = o.desc(&desc); }
                    2 => { o = o.placeholder(&placeholder); }
                    3 => { if required { o = o.required(); } }
                    4 => { if let Some(ref d) = default { o = o.default(d); } }
                    5 => { if let Some(ref e) = env_var { o = o.env(e); } }
                    6 => { if multi { o = o.multi(); } }
                    7 => { if hidden { o = o.hidden(); } }
                    _ => unreachable!(),
                }
            }
            OptionDef::from(o)
        };

        // Test a representative sample of permutations (all 8! = 40320 is too many).
        // Use forward, reverse, and several shuffled orderings.
        let orderings: &[&[usize]] = &[
            &[0, 1, 2, 3, 4, 5, 6, 7],
            &[7, 6, 5, 4, 3, 2, 1, 0],
            &[3, 7, 1, 5, 0, 2, 6, 4],
            &[6, 4, 2, 0, 7, 5, 3, 1],
            &[1, 0, 3, 2, 5, 4, 7, 6],
            &[5, 3, 7, 1, 4, 6, 0, 2],
        ];

        let baseline = apply(orderings[0]);
        for ordering in &orderings[1..] {
            prop_assert_eq!(&baseline, &apply(ordering));
        }
    }
}

// **Property 6: Pos modifier order independence with desc**
// **Validates: Requirements 4.3**
//
// For any valid positional name, optional description, and required flag,
// calling `.required()` and `.desc()` on `Pos::new(name)` in any permutation
// shall produce the same `PositionalDef`.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop6_pos_modifier_order_independence(
        name in arb_identifier(),
        desc in arb_description(),
        required in any::<bool>(),
    ) {
        // 0 = desc, 1 = required
        let apply = |order: [usize; 2]| -> PositionalDef {
            let mut p = Pos::new(&name);
            for &i in &order {
                match i {
                    0 => { p = p.desc(&desc); }
                    1 => { if required { p = p.required(); } }
                    _ => unreachable!(),
                }
            }
            PositionalDef::from(p)
        };

        let perm_a = apply([0, 1]);
        let perm_b = apply([1, 0]);
        prop_assert_eq!(&perm_a, &perm_b);
    }
}
