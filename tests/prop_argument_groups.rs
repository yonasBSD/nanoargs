// Feature: argument-groups
// Property tests for argument groups and conflict sets

mod common;

use nanoargs::{ArgBuilder, Flag, Opt};
use proptest::prelude::*;

use common::arb_safe_identifier;

/// Generate 2–5 distinct identifiers for use as flag/option long names.
fn arb_distinct_names(count: std::ops::RangeInclusive<usize>) -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(arb_safe_identifier(), count).prop_map(|names| {
        let mut seen = std::collections::HashSet::new();
        names.into_iter().filter(|n| n != "help" && n != "version" && seen.insert(n.clone())).collect::<Vec<_>>()
    })
}

// Feature: argument-groups, Property 1: Builder stores groups and conflicts
// **Validates: Requirements 1.1, 1.4, 3.1, 3.4**
proptest! {
    #[test]
    fn prop1_builder_stores_groups_and_conflicts(
        names in arb_distinct_names(4..=8),
        group_name in arb_safe_identifier(),
        conflict_name in arb_safe_identifier(),
    ) {
        // Need at least 4 distinct names: 2 for group members, 2 for conflict members
        prop_assume!(names.len() >= 4);

        let (group_members, rest) = names.split_at(2);
        let conflict_members = &rest[..2];

        // Register all names as flags
        let mut builder = ArgBuilder::new();
        for name in &names {
            builder = builder.flag(Flag::new(name));
        }

        let gm: Vec<&str> = group_members.iter().map(|s| s.as_str()).collect();
        let cm: Vec<&str> = conflict_members.iter().map(|s| s.as_str()).collect();

        let parser = builder
            .group(&group_name, &gm)
            .conflict(&conflict_name, &cm)
            .build()
            .unwrap();

        // Verify group stored correctly
        prop_assert_eq!(parser.groups().len(), 1);
        prop_assert_eq!(&parser.groups()[0].name, &group_name);
        prop_assert_eq!(&parser.groups()[0].members, &group_members.to_vec());

        // Verify conflict stored correctly
        prop_assert_eq!(parser.conflicts().len(), 1);
        prop_assert_eq!(&parser.conflicts()[0].name, &conflict_name);
        prop_assert_eq!(&parser.conflicts()[0].members, &conflict_members.to_vec());
    }
}

// Feature: argument-groups, Property 1 (multiple sets): Builder stores multiple groups and conflicts independently
proptest! {
    #[test]
    fn prop1b_builder_stores_multiple_groups_independently(
        names in arb_distinct_names(6..=10),
        group1_name in arb_safe_identifier(),
        group2_name in arb_safe_identifier(),
    ) {
        prop_assume!(names.len() >= 6);

        let members1 = &names[0..2];
        let members2 = &names[2..4];

        let mut builder = ArgBuilder::new();
        for name in &names {
            builder = builder.flag(Flag::new(name));
        }

        let m1: Vec<&str> = members1.iter().map(|s| s.as_str()).collect();
        let m2: Vec<&str> = members2.iter().map(|s| s.as_str()).collect();

        let parser = builder
            .group(&group1_name, &m1)
            .group(&group2_name, &m2)
            .build()
            .unwrap();

        prop_assert_eq!(parser.groups().len(), 2);
        prop_assert_eq!(&parser.groups()[0].name, &group1_name);
        prop_assert_eq!(&parser.groups()[1].name, &group2_name);
    }
}

// Feature: argument-groups, Property 2: Unknown member names cause build failure
// **Validates: Requirements 1.3, 3.3**
proptest! {
    #[test]
    fn prop2_unknown_group_member_causes_build_failure(
        known_name in arb_safe_identifier(),
        unknown_name in arb_safe_identifier(),
        group_name in arb_safe_identifier(),
    ) {
        prop_assume!(known_name != unknown_name);
        prop_assume!(known_name != "help" && known_name != "version");
        prop_assume!(unknown_name != "help" && unknown_name != "version");

        let result = ArgBuilder::new()
            .flag(Flag::new(&known_name))
            .group(&group_name, &[&known_name, &unknown_name])
            .build();

        prop_assert!(result.is_err());
        let err = result.unwrap_err();
        let msg = format!("{}", err);
        prop_assert!(msg.contains(&unknown_name), "Error should mention unknown member: {}", msg);
    }

    #[test]
    fn prop2_unknown_conflict_member_causes_build_failure(
        known_name in arb_safe_identifier(),
        unknown_name in arb_safe_identifier(),
        conflict_name in arb_safe_identifier(),
    ) {
        prop_assume!(known_name != unknown_name);
        prop_assume!(known_name != "help" && known_name != "version");
        prop_assume!(unknown_name != "help" && unknown_name != "version");

        let result = ArgBuilder::new()
            .flag(Flag::new(&known_name))
            .conflict(&conflict_name, &[&known_name, &unknown_name])
            .build();

        prop_assert!(result.is_err());
        let err = result.unwrap_err();
        let msg = format!("{}", err);
        prop_assert!(msg.contains(&unknown_name), "Error should mention unknown member: {}", msg);
    }
}

// Property 2: Too few members cause build failure
proptest! {
    #[test]
    fn prop2_too_few_group_members_causes_build_failure(
        name in arb_safe_identifier(),
        group_name in arb_safe_identifier(),
    ) {
        prop_assume!(name != "help" && name != "version");

        let result = ArgBuilder::new()
            .flag(Flag::new(&name))
            .group(&group_name, &[&name as &str])
            .build();

        prop_assert!(result.is_err());
    }

    #[test]
    fn prop2_too_few_conflict_members_causes_build_failure(
        name in arb_safe_identifier(),
        conflict_name in arb_safe_identifier(),
    ) {
        prop_assume!(name != "help" && name != "version");

        let result = ArgBuilder::new()
            .flag(Flag::new(&name))
            .conflict(&conflict_name, &[&name as &str])
            .build();

        prop_assert!(result.is_err());
    }
}

// Property 1: Groups work with options too, not just flags
proptest! {
    #[test]
    fn prop1_groups_work_with_options(
        opt1 in arb_safe_identifier(),
        opt2 in arb_safe_identifier(),
        group_name in arb_safe_identifier(),
    ) {
        prop_assume!(opt1 != opt2);
        prop_assume!(opt1 != "help" && opt1 != "version");
        prop_assume!(opt2 != "help" && opt2 != "version");

        let parser = ArgBuilder::new()
            .option(Opt::new(&opt1))
            .option(Opt::new(&opt2))
            .group(&group_name, &[&opt1 as &str, &opt2 as &str])
            .build()
            .unwrap();

        prop_assert_eq!(parser.groups().len(), 1);
        prop_assert_eq!(&parser.groups()[0].members, &vec![opt1, opt2]);
    }
}

// Feature: argument-groups, Property 9: Subcommand-level groups and conflicts are independent
// **Validates: Requirements 7.1, 7.2**
proptest! {
    #[test]
    fn prop9_subcommand_group_validated_against_subcommand_args(
        global_flag in arb_safe_identifier(),
        sub_flag1 in arb_safe_identifier(),
        sub_flag2 in arb_safe_identifier(),
        subcmd_name in "[a-z][a-z0-9]{1,9}".prop_filter("avoid help", |s| s != "help"),
        group_name in arb_safe_identifier(),
        provide_sub_flag in prop::sample::select(vec![0u8, 1, 2]),
    ) {
        prop_assume!(global_flag != "help" && global_flag != "version");
        prop_assume!(sub_flag1 != "help" && sub_flag1 != "version");
        prop_assume!(sub_flag2 != "help" && sub_flag2 != "version");
        prop_assume!(sub_flag1 != sub_flag2);
        prop_assume!(global_flag != sub_flag1 && global_flag != sub_flag2);

        // Subcommand parser has a group requiring at least one of sub_flag1 or sub_flag2
        let sub_parser = ArgBuilder::new()
            .flag(Flag::new(&sub_flag1))
            .flag(Flag::new(&sub_flag2))
            .group(&group_name, &[&sub_flag1, &sub_flag2])
            .build()
            .unwrap();

        // Parent parser has a global flag but NO groups/conflicts
        let parent = ArgBuilder::new()
            .flag(Flag::new(&global_flag))
            .subcommand(&subcmd_name, "test sub", sub_parser)
            .build()
            .unwrap();

        // Always provide the global flag, vary subcommand flags
        let mut args = vec![format!("--{global_flag}"), subcmd_name.clone()];
        match provide_sub_flag {
            0 => {} // provide neither sub flag → should fail with GroupViolation
            1 => args.push(format!("--{sub_flag1}")),
            _ => args.push(format!("--{sub_flag2}")),
        }

        let result = parent.parse(args);
        if provide_sub_flag == 0 {
            // Group violation from subcommand parser
            let err = result.unwrap_err();
            let msg = format!("{}", err);
            prop_assert!(msg.contains(&group_name), "Error should mention group name: {}", msg);
        } else {
            // Should succeed
            prop_assert!(result.is_ok(), "Expected success but got: {:?}", result);
        }
    }

    #[test]
    fn prop9_subcommand_conflict_validated_against_subcommand_args(
        global_flag in arb_safe_identifier(),
        sub_flag1 in arb_safe_identifier(),
        sub_flag2 in arb_safe_identifier(),
        subcmd_name in "[a-z][a-z0-9]{1,9}".prop_filter("avoid help", |s| s != "help"),
        conflict_name in arb_safe_identifier(),
        provide_count in 0u8..3,
    ) {
        prop_assume!(global_flag != "help" && global_flag != "version");
        prop_assume!(sub_flag1 != "help" && sub_flag1 != "version");
        prop_assume!(sub_flag2 != "help" && sub_flag2 != "version");
        prop_assume!(sub_flag1 != sub_flag2);
        prop_assume!(global_flag != sub_flag1 && global_flag != sub_flag2);

        // Subcommand parser has a conflict set between sub_flag1 and sub_flag2
        let sub_parser = ArgBuilder::new()
            .flag(Flag::new(&sub_flag1))
            .flag(Flag::new(&sub_flag2))
            .conflict(&conflict_name, &[&sub_flag1, &sub_flag2])
            .build()
            .unwrap();

        // Parent parser has a global flag but NO conflicts
        let parent = ArgBuilder::new()
            .flag(Flag::new(&global_flag))
            .subcommand(&subcmd_name, "test sub", sub_parser)
            .build()
            .unwrap();

        // Always provide the global flag, vary subcommand flags
        let mut args = vec![format!("--{global_flag}"), subcmd_name.clone()];
        if provide_count >= 1 {
            args.push(format!("--{sub_flag1}"));
        }
        if provide_count >= 2 {
            args.push(format!("--{sub_flag2}"));
        }

        let result = parent.parse(args);
        if provide_count >= 2 {
            // Conflict violation from subcommand parser
            let err = result.unwrap_err();
            let msg = format!("{}", err);
            prop_assert!(msg.contains(&conflict_name), "Error should mention conflict name: {}", msg);
        } else {
            // Should succeed
            prop_assert!(result.is_ok(), "Expected success but got: {:?}", result);
        }
    }

    #[test]
    fn prop9_parent_group_independent_of_subcommand(
        parent_flag1 in arb_safe_identifier(),
        parent_flag2 in arb_safe_identifier(),
        sub_flag in arb_safe_identifier(),
        subcmd_name in "[a-z][a-z0-9]{1,9}".prop_filter("avoid help", |s| s != "help"),
        group_name in arb_safe_identifier(),
        provide_parent_flag in prop::sample::select(vec![0u8, 1, 2]),
    ) {
        prop_assume!(parent_flag1 != "help" && parent_flag1 != "version");
        prop_assume!(parent_flag2 != "help" && parent_flag2 != "version");
        prop_assume!(sub_flag != "help" && sub_flag != "version");
        prop_assume!(parent_flag1 != parent_flag2);
        prop_assume!(sub_flag != parent_flag1 && sub_flag != parent_flag2);

        // Subcommand parser has no groups/conflicts
        let sub_parser = ArgBuilder::new()
            .flag(Flag::new(&sub_flag))
            .build()
            .unwrap();

        // Parent parser has a group requiring at least one of parent_flag1 or parent_flag2
        let parent = ArgBuilder::new()
            .flag(Flag::new(&parent_flag1))
            .flag(Flag::new(&parent_flag2))
            .group(&group_name, &[&parent_flag1, &parent_flag2])
            .subcommand(&subcmd_name, "test sub", sub_parser)
            .build()
            .unwrap();

        // Always provide the sub flag, vary parent flags
        let mut args = Vec::new();
        match provide_parent_flag {
            1 => args.push(format!("--{parent_flag1}")),
            2 => args.push(format!("--{parent_flag2}")),
            _ => {} // provide no parent flag → should fail with GroupViolation
        }
        args.push(subcmd_name.clone());
        args.push(format!("--{sub_flag}"));

        let result = parent.parse(args);
        if provide_parent_flag == 0 {
            // Group violation from parent parser (global args)
            let err = result.unwrap_err();
            let msg = format!("{}", err);
            prop_assert!(msg.contains(&group_name), "Error should mention group name: {}", msg);
        } else {
            prop_assert!(result.is_ok(), "Expected success but got: {:?}", result);
        }
    }
}
