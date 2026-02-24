#![allow(dead_code)]

use proptest::prelude::*;

pub fn arb_identifier() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9]{0,9}".prop_map(|s| s)
}

pub fn arb_short() -> impl Strategy<Value = Option<char>> {
    prop_oneof![
        Just(None),
        any::<u32>().prop_map(|v| Some((b'a' + (v % 26) as u8) as char))
    ]
}

pub fn arb_description() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 ]{1,30}"
}

pub fn arb_flag_def() -> impl Strategy<Value = (String, Option<char>, String)> {
    (arb_identifier(), arb_short(), arb_description())
}

pub fn arb_option_def() -> impl Strategy<Value = (String, Option<char>, String, String, bool, Option<String>)> {
    (
        arb_identifier(),
        arb_short(),
        arb_identifier(),
        arb_description(),
        any::<bool>(),
        prop_oneof![Just(None), arb_identifier().prop_map(Some),],
    )
}

pub fn arb_positional_def() -> impl Strategy<Value = (String, String, bool)> {
    (arb_identifier(), arb_description(), any::<bool>())
}

pub fn arb_safe_identifier() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9]{0,9}"
}

pub fn arb_safe_description() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 ]{1,20}"
}

pub fn arb_safe_option_def() -> impl Strategy<Value = (String, Option<char>, String, String, bool, Option<String>)> {
    (
        arb_safe_identifier(),
        arb_short(),
        arb_safe_identifier(),
        arb_safe_description(),
        any::<bool>(),
        prop_oneof![Just(None), arb_safe_identifier().prop_map(Some),],
    )
}

pub fn arb_safe_positional_def() -> impl Strategy<Value = (String, String, bool)> {
    (arb_safe_identifier(), arb_safe_description(), any::<bool>())
}

pub fn arb_safe_flag_def() -> impl Strategy<Value = (String, Option<char>, String)> {
    (arb_safe_identifier(), arb_short(), arb_safe_description())
}

pub fn arb_subcommand_name() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9]{1,9}".prop_filter("avoid help collision", |s| s != "help")
}

pub fn arb_safe_subcommand_def() -> impl Strategy<
    Value = (
        String,
        String,
        Vec<(String, Option<char>, String)>,
        Vec<(String, Option<char>, String, String, bool, Option<String>)>,
        Vec<(String, String, bool)>,
    ),
> {
    (
        arb_subcommand_name(),
        arb_safe_description(),
        prop::collection::vec(arb_safe_flag_def(), 0..3),
        prop::collection::vec(arb_safe_option_def(), 0..3),
        prop::collection::vec(arb_safe_positional_def(), 0..3),
    )
}

/// Generator: 1–6 distinct short-form chars from a-z excluding 'h' (reserved for help).
pub fn arb_flag_shorts(count: std::ops::RangeInclusive<usize>) -> impl Strategy<Value = Vec<char>> {
    let chars: Vec<char> = ('a'..='z').filter(|&c| c != 'h').collect();
    prop::collection::vec(prop::sample::select(chars), count)
        .prop_map(|v| {
            let mut seen = std::collections::HashSet::new();
            v.into_iter().filter(|c| seen.insert(*c)).collect::<Vec<_>>()
        })
        .prop_filter("need at least 1 distinct flag char", |v| !v.is_empty())
}

/// Generator: non-empty alphanumeric value string (no leading '-').
pub fn arb_value_string() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9]{1,12}"
}

/// Extracts lines belonging to a named section (e.g., "Flags:") from help text.
/// When the `color` feature is enabled, ANSI codes are stripped before matching.
pub fn extract_section_lines<'a>(help: &'a str, header: &str) -> Vec<&'a str> {
    // When color feature is enabled, section headers contain ANSI codes,
    // so we need to check stripped versions of lines for the header.
    let mut in_section = false;
    let mut lines = Vec::new();
    for line in help.lines() {
        let plain = strip_ansi_inline(line);
        if plain.trim() == header {
            in_section = true;
            continue;
        }
        if in_section {
            if line.is_empty() || (!plain.starts_with(' ') && !line.is_empty()) {
                break;
            }
            lines.push(line);
        }
    }
    lines
}

/// Inline ANSI stripping helper (always available, no-op when no ANSI present).
pub fn strip_ansi_inline(s: &str) -> String {
    let mut out = String::new();
    let mut in_escape = false;
    for ch in s.chars() {
        if in_escape {
            if ch.is_ascii_alphabetic() {
                in_escape = false;
            }
        } else if ch == '\x1b' {
            in_escape = true;
        } else {
            out.push(ch);
        }
    }
    out
}

/// Generator: valid environment variable name (uppercase alphanumeric with underscores).
pub fn arb_env_var_name() -> impl Strategy<Value = String> {
    "[A-Z][A-Z0-9_]{0,14}"
}
/// Generator: a `u32` value converted to string (always parseable as u32).
pub fn arb_u32_string() -> impl Strategy<Value = String> {
    any::<u32>().prop_map(|n| n.to_string())
}

/// Generator: a string that cannot be parsed as `u32` (contains at least one non-digit character).
pub fn arb_non_numeric_string() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9]*".prop_filter("must not parse as u32", |s| s.parse::<u32>().is_err())
}

// --- Color test helpers (only available with the `color` feature) ---

#[cfg(feature = "color")]
pub fn strip_ansi(s: &str) -> String {
    let mut out = String::new();
    let mut in_escape = false;
    for ch in s.chars() {
        if in_escape {
            if ch.is_ascii_alphabetic() {
                in_escape = false;
            }
        } else if ch == '\x1b' {
            in_escape = true;
        } else {
            out.push(ch);
        }
    }
    out
}

#[cfg(feature = "color")]
pub fn contains_ansi(s: &str) -> bool {
    s.contains('\x1b')
}

/// Check if `text` appears wrapped in green ANSI codes: \x1b[32m{text}\x1b[0m
#[cfg(feature = "color")]
pub fn contains_green(s: &str, text: &str) -> bool {
    s.contains(&format!("\x1b[32m{text}\x1b[0m"))
}

/// Check if `text` appears wrapped in cyan ANSI codes: \x1b[36m{text}\x1b[0m
#[cfg(feature = "color")]
pub fn contains_cyan(s: &str, text: &str) -> bool {
    s.contains(&format!("\x1b[36m{text}\x1b[0m"))
}

/// Check if `text` appears wrapped in dim ANSI codes: \x1b[2m{text}\x1b[0m
#[cfg(feature = "color")]
pub fn contains_dim(s: &str, text: &str) -> bool {
    s.contains(&format!("\x1b[2m{text}\x1b[0m"))
}

/// Check if `text` appears wrapped in bold+yellow ANSI codes: \x1b[1;33m{text}\x1b[0m
#[cfg(feature = "color")]
pub fn contains_bold_yellow(s: &str, text: &str) -> bool {
    s.contains(&format!("\x1b[1;33m{text}\x1b[0m"))
}

/// Check if `text` appears wrapped in bold+red ANSI codes: \x1b[1;31m{text}\x1b[0m
#[cfg(feature = "color")]
pub fn contains_bold_red(s: &str, text: &str) -> bool {
    s.contains(&format!("\x1b[1;31m{text}\x1b[0m"))
}

/// Check if `text` appears wrapped in yellow ANSI codes: \x1b[33m{text}\x1b[0m
#[cfg(feature = "color")]
pub fn contains_yellow(s: &str, text: &str) -> bool {
    s.contains(&format!("\x1b[33m{text}\x1b[0m"))
}
