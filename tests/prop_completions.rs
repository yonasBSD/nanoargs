mod common;

use common::*;
use nanoargs::{ArgBuilder, Flag, Opt, Shell};
use proptest::prelude::*;
use std::collections::HashSet;

// ── Strategies ─────────────────────────────────────────────────────────────

/// Strategy for a random Shell variant.
fn arb_shell() -> impl Strategy<Value = Shell> {
    prop_oneof![
        Just(Shell::Bash),
        Just(Shell::Zsh),
        Just(Shell::Fish),
        Just(Shell::PowerShell),
    ]
}

/// Strategy for a flag definition tuple: (long, short, description, hidden).
fn arb_completion_flag() -> impl Strategy<Value = (String, Option<char>, String, bool)> {
    (
        arb_safe_identifier(),
        arb_short(),
        arb_safe_description(),
        any::<bool>(),
    )
}

/// Strategy for an option definition tuple: (long, short, placeholder, description, hidden).
fn arb_completion_option() -> impl Strategy<Value = (String, Option<char>, String, String, bool)> {
    (
        arb_safe_identifier(),
        arb_short(),
        arb_safe_identifier(),
        arb_safe_description(),
        any::<bool>(),
    )
}

/// Build an ArgParser from generated parts, deduplicating long names and short chars.
fn build_parser(
    name: Option<String>,
    flags: &[(String, Option<char>, String, bool)],
    options: &[(String, Option<char>, String, String, bool)],
    subcommands: &[(
        String,
        String,
        Vec<(String, Option<char>, String, bool)>,
        Vec<(String, Option<char>, String, String, bool)>,
    )],
) -> Option<nanoargs::ArgParser> {
    let mut builder = ArgBuilder::new();
    if let Some(n) = &name {
        builder = builder.name(n);
    }

    let mut longs = HashSet::new();
    let mut shorts = HashSet::new();
    longs.insert("help".to_string());
    shorts.insert('h');

    for (long, short, desc, hidden) in flags {
        if !longs.insert(long.clone()) {
            continue;
        }
        let mut f = Flag::new(long).desc(desc);
        if let Some(c) = short {
            if shorts.insert(*c) {
                f = f.short(*c);
            }
        }
        if *hidden {
            f = f.hidden();
        }
        builder = builder.flag(f);
    }

    for (long, short, placeholder, desc, hidden) in options {
        if !longs.insert(long.clone()) {
            continue;
        }
        let mut o = Opt::new(long).placeholder(placeholder).desc(desc);
        if let Some(c) = short {
            if shorts.insert(*c) {
                o = o.short(*c);
            }
        }
        if *hidden {
            o = o.hidden();
        }
        builder = builder.option(o);
    }

    let mut sub_names = HashSet::new();
    for (sub_name, sub_desc, sub_flags, sub_options) in subcommands {
        if !sub_names.insert(sub_name.clone()) {
            continue;
        }
        let mut sub_builder = ArgBuilder::new();
        let mut sub_longs = HashSet::new();
        let mut sub_shorts = HashSet::new();
        sub_longs.insert("help".to_string());
        sub_shorts.insert('h');

        for (long, short, desc, hidden) in sub_flags {
            if !sub_longs.insert(long.clone()) {
                continue;
            }
            let mut f = Flag::new(long).desc(desc);
            if let Some(c) = short {
                if sub_shorts.insert(*c) {
                    f = f.short(*c);
                }
            }
            if *hidden {
                f = f.hidden();
            }
            sub_builder = sub_builder.flag(f);
        }

        for (long, short, placeholder, desc, hidden) in sub_options {
            if !sub_longs.insert(long.clone()) {
                continue;
            }
            let mut o = Opt::new(long).placeholder(placeholder).desc(desc);
            if let Some(c) = short {
                if sub_shorts.insert(*c) {
                    o = o.short(*c);
                }
            }
            if *hidden {
                o = o.hidden();
            }
            sub_builder = sub_builder.option(o);
        }

        let sub_parser = sub_builder.build().ok()?;
        builder = builder.subcommand(sub_name, sub_desc, sub_parser);
    }

    builder.build().ok()
}

/// Strategy for subcommand definitions with nested flags/options.
fn arb_completion_subcommand() -> impl Strategy<
    Value = (
        String,
        String,
        Vec<(String, Option<char>, String, bool)>,
        Vec<(String, Option<char>, String, String, bool)>,
    ),
> {
    (
        arb_subcommand_name(),
        arb_safe_description(),
        prop::collection::vec(arb_completion_flag(), 0..3),
        prop::collection::vec(arb_completion_option(), 0..3),
    )
}

// ── Property 1: Non-empty output for any valid parser and shell ────────────
// Feature: shell-completions, Property 1: Non-empty output for any valid parser and shell
// **Validates: Requirements 1.2**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    #[test]
    fn prop_completions_1_non_empty_output(
        name in prop::option::of(arb_safe_identifier()),
        flags in prop::collection::vec(arb_completion_flag(), 1..4),
        options in prop::collection::vec(arb_completion_option(), 0..4),
        subcommands in prop::collection::vec(arb_completion_subcommand(), 0..3),
        shell in arb_shell(),
    ) {
        if let Some(parser) = build_parser(name, &flags, &options, &subcommands) {
            // Ensure there's at least one non-hidden item for Fish (which only emits per-item lines)
            let has_visible = parser.flags().iter().any(|f| !f.hidden)
                || parser.options().iter().any(|o| !o.hidden)
                || !parser.subcommands().is_empty();
            if !has_visible && shell == Shell::Fish {
                // Fish produces empty output for parsers with only hidden items — skip
                return Ok(());
            }
            let output = parser.generate_completions(shell);
            prop_assert!(!output.is_empty(), "Completion output should never be empty");
        }
    }
}

/// Check that a short char appears in the output for the given shell.
/// Fish uses `-s <char>`, others use `-<char>`.
fn output_contains_short(output: &str, ch: char, shell: Shell) -> bool {
    match shell {
        Shell::Fish => output.contains(&format!("-s {ch}")),
        _ => output.contains(&format!("-{ch}")),
    }
}

/// Check that `--<long>` appears as a distinct token (not as a prefix of another flag).
/// Fish uses `-l <long>` syntax instead of `--<long>`.
fn output_contains_long_flag(output: &str, long: &str, shell: Shell) -> bool {
    match shell {
        Shell::Fish => {
            // Fish uses `-l <long>` syntax; check word boundary after the long name
            let needle = format!("-l {long}");
            for (i, _) in output.match_indices(&needle) {
                let after = i + needle.len();
                if after >= output.len() {
                    return true;
                }
                let next_ch = output[after..].chars().next().unwrap();
                if !next_ch.is_alphanumeric() && next_ch != '_' && next_ch != '-' {
                    return true;
                }
            }
            false
        }
        _ => {
            let needle = format!("--{long}");
            for (i, _) in output.match_indices(&needle) {
                let after = i + needle.len();
                if after >= output.len() {
                    return true;
                }
                let next_ch = output[after..].chars().next().unwrap();
                if !next_ch.is_alphanumeric() && next_ch != '_' && next_ch != '-' {
                    return true;
                }
            }
            false
        }
    }
}

// ── Property 2: Non-hidden flags appear with long and short forms ──────────
// Feature: shell-completions, Property 2: Non-hidden flags appear with long and short forms
// **Validates: Requirements 2.1, 2.2**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    #[test]
    fn prop_completions_2_non_hidden_flags_appear(
        name in Just(Some("testprog".to_string())),
        flags in prop::collection::vec(arb_completion_flag(), 1..5),
        options in prop::collection::vec(arb_completion_option(), 0..2),
        subcommands in prop::collection::vec(arb_completion_subcommand(), 0..2),
        shell in arb_shell(),
    ) {
        if let Some(parser) = build_parser(name, &flags, &options, &subcommands) {
            let output = parser.generate_completions(shell);
            for flag in parser.flags() {
                if flag.hidden {
                    continue;
                }
                prop_assert!(
                    output.contains(&flag.long),
                    "Output for {:?} missing non-hidden flag long name '{}'\nOutput:\n{}",
                    shell, flag.long, output
                );
                if let Some(c) = flag.short {
                    prop_assert!(
                        output_contains_short(&output, c, shell),
                        "Output for {:?} missing short form '-{}' for flag '{}'\nOutput:\n{}",
                        shell, c, flag.long, output
                    );
                }
            }
        }
    }
}

// ── Property 3: Non-hidden options appear with long and short forms ────────
// Feature: shell-completions, Property 3: Non-hidden options appear with long and short forms
// **Validates: Requirements 3.1, 3.2**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    #[test]
    fn prop_completions_3_non_hidden_options_appear(
        name in Just(Some("testprog".to_string())),
        flags in prop::collection::vec(arb_completion_flag(), 0..2),
        options in prop::collection::vec(arb_completion_option(), 1..5),
        subcommands in prop::collection::vec(arb_completion_subcommand(), 0..2),
        shell in arb_shell(),
    ) {
        if let Some(parser) = build_parser(name, &flags, &options, &subcommands) {
            let output = parser.generate_completions(shell);
            for opt in parser.options() {
                if opt.hidden {
                    continue;
                }
                prop_assert!(
                    output.contains(&opt.long),
                    "Output for {:?} missing non-hidden option long name '{}'\nOutput:\n{}",
                    shell, opt.long, output
                );
                if let Some(c) = opt.short {
                    prop_assert!(
                        output_contains_short(&output, c, shell),
                        "Output for {:?} missing short form '-{}' for option '{}'\nOutput:\n{}",
                        shell, c, opt.long, output
                    );
                }
            }
        }
    }
}

// ── Property 4: Options indicate they expect a value ───────────────────────
// Feature: shell-completions, Property 4: Options indicate they expect a value
// **Validates: Requirements 3.3**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    #[test]
    fn prop_completions_4_options_expect_value(
        name in Just(Some("testprog".to_string())),
        flags in prop::collection::vec(arb_completion_flag(), 0..2),
        options in prop::collection::vec(arb_completion_option(), 1..4),
        shell in arb_shell(),
    ) {
        if let Some(parser) = build_parser(name, &flags, &options, &[]) {
            let output = parser.generate_completions(shell);
            for opt in parser.options() {
                if opt.hidden {
                    continue;
                }
                let has_value_indicator = match shell {
                    Shell::Zsh => {
                        // Zsh uses ':placeholder:' after the option spec
                        let pattern = format!("--{}[", opt.long);
                        if let Some(pos) = output.find(&pattern) {
                            // After the closing ']' there should be ':...:' indicating value
                            let after = &output[pos..];
                            after.contains("]:") // closing bracket followed by colon
                        } else {
                            false
                        }
                    }
                    Shell::Fish => {
                        // Fish uses -r flag for options requiring a value
                        // Find the line with this option's long name
                        output.lines().any(|line| {
                            line.contains(&format!("-l {}", opt.long)) && line.contains(" -r")
                        })
                    }
                    Shell::Bash => {
                        // Bash: options are included in the completable words
                        // (the design separates them so the completion function knows)
                        output.contains(&format!("--{}", opt.long))
                    }
                    Shell::PowerShell => {
                        // PowerShell: options appear as CompletionResult entries
                        output.contains(&format!("--{}", opt.long))
                    }
                };
                prop_assert!(
                    has_value_indicator,
                    "Output for {:?} missing value indicator for option '{}'\nOutput:\n{}",
                    shell, opt.long, output
                );
            }
        }
    }
}

// ── Property 5: Hidden flags and options are excluded ──────────────────────
// Feature: shell-completions, Property 5: Hidden flags and options are excluded
// **Validates: Requirements 2.4, 3.5**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    #[test]
    fn prop_completions_5_hidden_excluded(
        _name in Just(Some("testprog".to_string())),
        shell in arb_shell(),
        visible_flag in arb_safe_identifier().prop_filter("not help", |s| s != "help"),
        hidden_flag in arb_safe_identifier().prop_filter("not help", |s| s != "help"),
        visible_opt in arb_safe_identifier().prop_filter("not help", |s| s != "help"),
        hidden_opt in arb_safe_identifier().prop_filter("not help", |s| s != "help"),
    ) {
        // Ensure all long names are distinct
        let names = [&visible_flag, &hidden_flag, &visible_opt, &hidden_opt];
        let unique: HashSet<&String> = names.iter().copied().collect();
        prop_assume!(unique.len() == 4);

        let parser = ArgBuilder::new()
            .name("testprog")
            .flag(Flag::new(&visible_flag).desc("visible"))
            .flag(Flag::new(&hidden_flag).desc("hidden").hidden())
            .option(Opt::new(&visible_opt).placeholder("VAL").desc("visible"))
            .option(Opt::new(&hidden_opt).placeholder("VAL").desc("hidden").hidden())
            .build()
            .unwrap();

        let output = parser.generate_completions(shell);

        // Hidden flags/options should NOT appear as --prefixed tokens
        prop_assert!(
            !output_contains_long_flag(&output, &hidden_flag, shell),
            "Output for {:?} should not contain hidden flag '--{}'\nOutput:\n{}",
            shell, hidden_flag, output
        );
        prop_assert!(
            !output_contains_long_flag(&output, &hidden_opt, shell),
            "Output for {:?} should not contain hidden option '--{}'\nOutput:\n{}",
            shell, hidden_opt, output
        );

        // Visible ones should appear
        prop_assert!(
            output_contains_long_flag(&output, &visible_flag, shell),
            "Output for {:?} should contain visible flag '--{}'\nOutput:\n{}",
            shell, visible_flag, output
        );
        prop_assert!(
            output_contains_long_flag(&output, &visible_opt, shell),
            "Output for {:?} should contain visible option '--{}'\nOutput:\n{}",
            shell, visible_opt, output
        );
    }
}

// ── Property 6: Descriptions appear for supporting shells ──────────────────
// Feature: shell-completions, Property 6: Descriptions appear for supporting shells
// **Validates: Requirements 2.3, 3.4, 4.2**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    #[test]
    fn prop_completions_6_descriptions_appear(
        name in Just(Some("testprog".to_string())),
        flags in prop::collection::vec(arb_completion_flag(), 1..3),
        options in prop::collection::vec(arb_completion_option(), 1..3),
        subcommands in prop::collection::vec(arb_completion_subcommand(), 0..2),
        shell in prop_oneof![Just(Shell::Zsh), Just(Shell::Fish), Just(Shell::PowerShell)],
    ) {
        if let Some(parser) = build_parser(name, &flags, &options, &subcommands) {
            let output = parser.generate_completions(shell);
            for flag in parser.flags() {
                if flag.hidden || flag.description.is_empty() {
                    continue;
                }
                // Descriptions may be escaped; check for the core alphanumeric content
                let desc_words: Vec<&str> = flag.description.split_whitespace().collect();
                if let Some(first_word) = desc_words.first() {
                    prop_assert!(
                        output.contains(first_word),
                        "Output for {:?} missing description word '{}' for flag '{}'\nDesc: '{}'\nOutput:\n{}",
                        shell, first_word, flag.long, flag.description, output
                    );
                }
            }
            for opt in parser.options() {
                if opt.hidden || opt.description.is_empty() {
                    continue;
                }
                let desc_words: Vec<&str> = opt.description.split_whitespace().collect();
                if let Some(first_word) = desc_words.first() {
                    prop_assert!(
                        output.contains(first_word),
                        "Output for {:?} missing description word '{}' for option '{}'\nDesc: '{}'\nOutput:\n{}",
                        shell, first_word, opt.long, opt.description, output
                    );
                }
            }
            for sub in parser.subcommands() {
                if sub.description.is_empty() {
                    continue;
                }
                let desc_words: Vec<&str> = sub.description.split_whitespace().collect();
                if let Some(first_word) = desc_words.first() {
                    prop_assert!(
                        output.contains(first_word),
                        "Output for {:?} missing description word '{}' for subcommand '{}'\nDesc: '{}'\nOutput:\n{}",
                        shell, first_word, sub.name, sub.description, output
                    );
                }
            }
        }
    }
}

// ── Property 7: Subcommand names appear in output ──────────────────────────
// Feature: shell-completions, Property 7: Subcommand names appear in output
// **Validates: Requirements 4.1**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    #[test]
    fn prop_completions_7_subcommand_names_appear(
        name in Just(Some("testprog".to_string())),
        flags in prop::collection::vec(arb_completion_flag(), 0..2),
        options in prop::collection::vec(arb_completion_option(), 0..2),
        subcommands in prop::collection::vec(arb_completion_subcommand(), 1..4),
        shell in arb_shell(),
    ) {
        if let Some(parser) = build_parser(name, &flags, &options, &subcommands) {
            let output = parser.generate_completions(shell);
            for sub in parser.subcommands() {
                prop_assert!(
                    output.contains(&sub.name),
                    "Output for {:?} missing subcommand name '{}'\nOutput:\n{}",
                    shell, sub.name, output
                );
            }
        }
    }
}

// ── Property 8: Subcommand-specific arguments appear in output ─────────────
// Feature: shell-completions, Property 8: Subcommand-specific arguments appear in output
// **Validates: Requirements 4.3**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    #[test]
    fn prop_completions_8_subcommand_specific_args(
        name in Just(Some("testprog".to_string())),
        subcommands in prop::collection::vec(arb_completion_subcommand(), 1..3),
        shell in arb_shell(),
    ) {
        if let Some(parser) = build_parser(name, &[], &[], &subcommands) {
            let output = parser.generate_completions(shell);
            for sub in parser.subcommands() {
                for flag in sub.parser.flags() {
                    if flag.hidden {
                        continue;
                    }
                    prop_assert!(
                        output.contains(&flag.long),
                        "Output for {:?} missing subcommand '{}' flag '{}'\nOutput:\n{}",
                        shell, sub.name, flag.long, output
                    );
                }
                for opt in sub.parser.options() {
                    if opt.hidden {
                        continue;
                    }
                    prop_assert!(
                        output.contains(&opt.long),
                        "Output for {:?} missing subcommand '{}' option '{}'\nOutput:\n{}",
                        shell, sub.name, opt.long, output
                    );
                }
            }
        }
    }
}

// ── Property 9: Program name appears in output ─────────────────────────────
// Feature: shell-completions, Property 9: Program name appears in output
// **Validates: Requirements 10.1**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    #[test]
    fn prop_completions_9_program_name_appears(
        name in "[a-z][a-z0-9]{2,9}",
        flags in prop::collection::vec(arb_completion_flag(), 1..3),
        options in prop::collection::vec(arb_completion_option(), 0..3),
        shell in arb_shell(),
    ) {
        if let Some(parser) = build_parser(Some(name.clone()), &flags, &options, &[]) {
            // Ensure there's at least one visible item for Fish
            let has_visible = parser.flags().iter().any(|f| !f.hidden)
                || parser.options().iter().any(|o| !o.hidden);
            if !has_visible && shell == Shell::Fish {
                return Ok(());
            }
            let output = parser.generate_completions(shell);
            prop_assert!(
                output.contains(&name),
                "Output for {:?} missing program name '{}'\nOutput:\n{}",
                shell, name, output
            );
        }
    }
}

// ── Property 10: Unrecognized shell names are rejected ─────────────────────
// Feature: shell-completions, Property 10: Unrecognized shell names are rejected
// **Validates: Requirements 1.4**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    #[test]
    fn prop_completions_10_unrecognized_shell_rejected(
        name in "[a-zA-Z]{1,15}".prop_filter(
            "must not be a recognized shell name",
            |s| {
                let lower = s.to_lowercase();
                lower != "bash" && lower != "zsh" && lower != "fish"
                    && lower != "powershell" && lower != "pwsh"
            }
        ),
    ) {
        let result: Result<Shell, String> = name.parse();
        prop_assert!(
            result.is_err(),
            "Shell::from_str should reject '{}' but got {:?}",
            name, result
        );
    }
}

// ── Unit Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod unit_tests {
    use nanoargs::{ArgBuilder, Flag, Opt, Shell};

    // ── Shell::from_str known variants ─────────────────────────────────────

    #[test]
    fn shell_from_str_bash() {
        assert_eq!("bash".parse::<Shell>().unwrap(), Shell::Bash);
        assert_eq!("BASH".parse::<Shell>().unwrap(), Shell::Bash);
    }

    #[test]
    fn shell_from_str_zsh() {
        assert_eq!("zsh".parse::<Shell>().unwrap(), Shell::Zsh);
        assert_eq!("ZSH".parse::<Shell>().unwrap(), Shell::Zsh);
    }

    #[test]
    fn shell_from_str_fish() {
        assert_eq!("fish".parse::<Shell>().unwrap(), Shell::Fish);
        assert_eq!("Fish".parse::<Shell>().unwrap(), Shell::Fish);
    }

    #[test]
    fn shell_from_str_powershell() {
        assert_eq!("powershell".parse::<Shell>().unwrap(), Shell::PowerShell);
        assert_eq!("pwsh".parse::<Shell>().unwrap(), Shell::PowerShell);
    }

    #[test]
    fn shell_from_str_unknown() {
        assert!("nushell".parse::<Shell>().is_err());
        assert!("".parse::<Shell>().is_err());
    }

    // ── Shell-specific structural markers ──────────────────────────────────

    fn sample_parser() -> nanoargs::ArgParser {
        ArgBuilder::new()
            .name("myapp")
            .flag(Flag::new("verbose").short('v').desc("Enable verbose"))
            .option(Opt::new("output").short('o').placeholder("FILE").desc("Output file"))
            .build()
            .unwrap()
    }

    #[test]
    fn bash_contains_complete_f() {
        let output = sample_parser().generate_completions(Shell::Bash);
        assert!(output.contains("complete -F"), "Bash output missing 'complete -F'");
    }

    #[test]
    fn zsh_contains_compdef() {
        let output = sample_parser().generate_completions(Shell::Zsh);
        assert!(output.contains("#compdef"), "Zsh output missing '#compdef'");
    }

    #[test]
    fn fish_contains_complete_c() {
        let output = sample_parser().generate_completions(Shell::Fish);
        assert!(output.contains("complete -c"), "Fish output missing 'complete -c'");
    }

    #[test]
    fn powershell_contains_register_argument_completer() {
        let output = sample_parser().generate_completions(Shell::PowerShell);
        assert!(
            output.contains("Register-ArgumentCompleter"),
            "PowerShell output missing 'Register-ArgumentCompleter'"
        );
    }

    // ── Default program name fallback ──────────────────────────────────────

    #[test]
    fn default_program_name_fallback() {
        let parser = ArgBuilder::new().flag(Flag::new("verbose").desc("verbose")).build().unwrap();

        for shell in [Shell::Bash, Shell::Zsh, Shell::Fish, Shell::PowerShell] {
            let output = parser.generate_completions(shell);
            assert!(
                output.contains("program"),
                "{:?} output should use 'program' as default name",
                shell
            );
        }
    }

    // ── Empty parser output ────────────────────────────────────────────────

    #[test]
    fn empty_parser_produces_valid_output() {
        let parser = ArgBuilder::new().name("empty").build().unwrap();

        // Bash, Zsh, PowerShell still produce structural output; Fish may be empty
        let bash = parser.generate_completions(Shell::Bash);
        assert!(bash.contains("complete -F"));

        let zsh = parser.generate_completions(Shell::Zsh);
        assert!(zsh.contains("#compdef"));

        let ps = parser.generate_completions(Shell::PowerShell);
        assert!(ps.contains("Register-ArgumentCompleter"));
    }

    // ── Special character escaping in descriptions ─────────────────────────

    #[test]
    fn bash_escapes_single_quotes() {
        let parser = ArgBuilder::new().name("app").flag(Flag::new("test").desc("it's a flag")).build().unwrap();
        let output = parser.generate_completions(Shell::Bash);
        // Single quote should be escaped as '\'' in bash
        assert!(
            !output.contains("it's"),
            "Bash should escape single quotes in descriptions"
        );
    }

    #[test]
    fn zsh_escapes_brackets_and_colons() {
        let parser = ArgBuilder::new().name("app").flag(Flag::new("test").desc("use [option]: value")).build().unwrap();
        let output = parser.generate_completions(Shell::Zsh);
        assert!(
            output.contains("\\[") && output.contains("\\]") && output.contains("\\:"),
            "Zsh should escape brackets and colons in descriptions"
        );
    }

    #[test]
    fn powershell_escapes_single_quotes() {
        let parser = ArgBuilder::new().name("app").flag(Flag::new("test").desc("it's a flag")).build().unwrap();
        let output = parser.generate_completions(Shell::PowerShell);
        // PowerShell escapes single quotes by doubling them
        assert!(
            output.contains("it''s"),
            "PowerShell should escape single quotes by doubling them"
        );
    }

    #[test]
    fn fish_escapes_single_quotes() {
        let parser = ArgBuilder::new().name("app").flag(Flag::new("test").desc("it's a flag")).build().unwrap();
        let output = parser.generate_completions(Shell::Fish);
        assert!(
            !output.contains("it's a"),
            "Fish should escape single quotes in descriptions"
        );
    }

    /// Regression: Zsh subcommand dispatch must not fall through to _describe.
    /// When a known subcommand is matched, its sub-function should handle
    /// completions and _describe (which lists subcommand names) should only
    /// appear inside a catch-all `*) ... ;;` branch.
    #[test]
    fn zsh_subcommand_dispatch_does_not_fall_through_to_describe() {
        let sub = ArgBuilder::new().name("sub").flag(Flag::new("flag").desc("a flag")).build().unwrap();
        let parser = ArgBuilder::new().name("app").subcommand("sub", "A subcommand", sub).build().unwrap();
        let output = parser.generate_completions(Shell::Zsh);

        // _describe must only appear inside a *) default branch, not after esac
        // of the subcommand case dispatch. Find the inner case block and verify
        // _describe is inside *), not after esac.
        let lines: Vec<&str> = output.lines().collect();
        let mut in_subcmd_case = false;
        let mut found_describe_in_default = false;
        let mut found_describe_outside = false;
        let mut in_default_branch = false;

        for line in &lines {
            let trimmed = line.trim();
            if trimmed.starts_with("case $line") {
                in_subcmd_case = true;
                continue;
            }
            if in_subcmd_case {
                if trimmed == "*)" {
                    in_default_branch = true;
                }
                if trimmed == "esac" {
                    in_subcmd_case = false;
                    in_default_branch = false;
                    continue;
                }
                if trimmed.contains("_describe") {
                    if in_default_branch {
                        found_describe_in_default = true;
                    } else {
                        found_describe_outside = true;
                    }
                }
            }
        }

        assert!(
            found_describe_in_default,
            "_describe should appear inside the *) default branch"
        );
        assert!(
            !found_describe_outside,
            "_describe should NOT appear outside the *) default branch (would cause fallthrough)"
        );
    }
}
