mod common;

use common::*;
use nanoargs::{ArgBuilder, Opt};
use proptest::prelude::*;

// Builder stores env_var correctly
proptest! {
    #[test]
    fn prop1_builder_stores_env_var(
        long in arb_identifier(),
        short in arb_short(),
        placeholder in arb_identifier(),
        desc in arb_description(),
        env_var in arb_env_var_name(),
        default_val in arb_identifier(),
        method in 0u32..6,
    ) {
        let o = match method {
            0 => {
                let mut o = Opt::new(&long).placeholder(&placeholder).desc(&desc).env(&env_var);
                if let Some(ch) = short { o = o.short(ch); }
                o
            }
            1 => {
                let mut o = Opt::new(&long).placeholder(&placeholder).desc(&desc).env(&env_var).required();
                if let Some(ch) = short { o = o.short(ch); }
                o
            }
            2 => {
                let mut o = Opt::new(&long).placeholder(&placeholder).desc(&desc).env(&env_var).default(&default_val);
                if let Some(ch) = short { o = o.short(ch); }
                o
            }
            3 => {
                let mut o = Opt::new(&long).placeholder(&placeholder).desc(&desc);
                if let Some(ch) = short { o = o.short(ch); }
                o
            }
            4 => {
                let mut o = Opt::new(&long).placeholder(&placeholder).desc(&desc).required();
                if let Some(ch) = short { o = o.short(ch); }
                o
            }
            5 => {
                let mut o = Opt::new(&long).placeholder(&placeholder).desc(&desc).default(&default_val);
                if let Some(ch) = short { o = o.short(ch); }
                o
            }
            _ => unreachable!(),
        };

        let parser = ArgBuilder::new().option(o).build().unwrap();
        let opt = &parser.options()[0];

        match method {
            0 | 1 | 2 => {
                prop_assert_eq!(&opt.env_var, &Some(env_var.clone()),
                    "option_env* method should store env_var");
            }
            3 | 4 | 5 => {
                prop_assert_eq!(&opt.env_var, &None,
                    "option/option_required/option_default should have env_var = None");
            }
            _ => unreachable!(),
        }

        prop_assert_eq!(&opt.long, &long);
        prop_assert_eq!(&opt.short, &short);
        prop_assert_eq!(&opt.placeholder, &placeholder);
        prop_assert_eq!(&opt.description, &desc);

        match method {
            0 => {
                prop_assert!(!opt.required);
                prop_assert_eq!(&opt.default, &None);
            }
            1 => {
                prop_assert!(opt.required);
                prop_assert_eq!(&opt.default, &None);
            }
            2 => {
                prop_assert!(!opt.required);
                prop_assert_eq!(&opt.default, &Some(default_val.clone()));
            }
            3 => {
                prop_assert!(!opt.required);
                prop_assert_eq!(&opt.default, &None);
            }
            4 => {
                prop_assert!(opt.required);
                prop_assert_eq!(&opt.default, &None);
            }
            5 => {
                prop_assert!(!opt.required);
                prop_assert_eq!(&opt.default, &Some(default_val.clone()));
            }
            _ => unreachable!(),
        }
    }
}

// Value resolution order: CLI > env var > default > required error > absent
proptest! {
    #[test]
    fn prop2_value_resolution_order(
        long in arb_identifier(),
        placeholder in arb_identifier(),
        desc in arb_description(),
        env_var_suffix in arb_env_var_name(),
        cli_value in arb_value_string(),
        env_value in arb_value_string(),
        default_val in arb_value_string(),
        provide_cli in any::<bool>(),
        env_state in 0u32..3,
        has_default in any::<bool>(),
        is_required in any::<bool>(),
        run_id in 0u64..u64::MAX,
    ) {
        let env_var_name = format!("NANOARGS_P2_{}_{}", run_id, env_var_suffix);

        match env_state {
            0 => std::env::remove_var(&env_var_name),
            1 => std::env::set_var(&env_var_name, ""),
            2 => std::env::set_var(&env_var_name, &env_value),
            _ => unreachable!(),
        }

        let mut o = Opt::new(&long).placeholder(&placeholder).desc(&desc).env(&env_var_name);
        if has_default {
            o = o.default(&default_val);
        } else if is_required {
            o = o.required();
        }

        let parser = ArgBuilder::new().option(o).build().unwrap();

        let args: Vec<String> = if provide_cli {
            vec![format!("--{}", long), cli_value.clone()]
        } else {
            vec![]
        };

        let result = parser.parse(args);
        std::env::remove_var(&env_var_name);

        if provide_cli {
            let r = result.unwrap();
            prop_assert_eq!(r.get_option(&long), Some(cli_value.as_str()));
        } else if env_state == 2 {
            let r = result.unwrap();
            prop_assert_eq!(r.get_option(&long), Some(env_value.as_str()));
        } else if has_default {
            let r = result.unwrap();
            prop_assert_eq!(r.get_option(&long), Some(default_val.as_str()));
        } else if is_required {
            match result {
                Err(nanoargs::ParseError::MissingRequired(name)) => {
                    prop_assert_eq!(name, long);
                }
                other => prop_assert!(false, "Expected MissingRequired, got {:?}", other),
            }
        } else {
            let r = result.unwrap();
            prop_assert_eq!(r.get_option(&long), None);
        }
    }
}

// Env var fallback works in subcommand context
proptest! {
    #[test]
    fn prop3_env_var_fallback_subcommand_context(
        global_long in arb_identifier(),
        sub_long in arb_identifier(),
        placeholder in arb_identifier(),
        desc in arb_description(),
        env_suffix in arb_env_var_name(),
        global_env_value in arb_value_string(),
        sub_env_value in arb_value_string(),
        sub_name in arb_subcommand_name(),
        run_id in 0u64..u64::MAX,
    ) {
        prop_assume!(global_long != sub_long);

        let global_env = format!("NANOARGS_P3G_{}_{}", run_id, env_suffix);
        let sub_env = format!("NANOARGS_P3S_{}_{}", run_id, env_suffix);

        std::env::set_var(&global_env, &global_env_value);
        std::env::set_var(&sub_env, &sub_env_value);

        let sub_parser = ArgBuilder::new()
            .option(Opt::new(&sub_long).placeholder(&placeholder).desc(&desc).env(&sub_env))
            .build()
            .unwrap();

        let parser = ArgBuilder::new()
            .option(Opt::new(&global_long).placeholder(&placeholder).desc(&desc).env(&global_env))
            .subcommand(&sub_name, &desc, sub_parser)
            .build()
            .unwrap();

        let args = vec![sub_name.clone()];
        let result = parser.parse(args);

        std::env::remove_var(&global_env);
        std::env::remove_var(&sub_env);

        let r = result.unwrap();

        prop_assert_eq!(r.get_option(&global_long), Some(global_env_value.as_str()),
            "Global option should use env var fallback");

        let sub_result = r.subcommand_result().unwrap();
        prop_assert_eq!(sub_result.get_option(&sub_long), Some(sub_env_value.as_str()),
            "Subcommand option should use env var fallback");
    }
}

// Help text shows [env: VAR] annotation iff env_var is configured
proptest! {
    #[test]
    fn prop4_help_text_env_var_annotation(
        long in arb_identifier(),
        short in arb_short(),
        placeholder in arb_identifier(),
        desc in arb_description(),
        env_var in arb_env_var_name(),
        has_env in any::<bool>(),
    ) {
        let mut o = Opt::new(&long).placeholder(&placeholder).desc(&desc);
        if let Some(ch) = short { o = o.short(ch); }
        if has_env { o = o.env(&env_var); }
        let parser = ArgBuilder::new().option(o).build().unwrap();
        let help = parser.help_text();

        let env_marker = format!("[env: {}]", env_var);

        if has_env {
            prop_assert!(help.contains(&env_marker),
                "Help text should contain '{}' when env_var is configured.\nHelp:\n{}", env_marker, help);
        } else {
            prop_assert!(!help.contains("[env: "),
                "Help text should NOT contain any [env: ...] marker when no env_var is configured.\nHelp:\n{}", help);
        }
    }
}
