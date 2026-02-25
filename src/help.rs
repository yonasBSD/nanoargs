use crate::parser::ArgParser;

/// Computes the visible character width of a string, excluding ANSI escape sequences.
#[cfg(feature = "color")]
pub(crate) fn visible_len(s: &str) -> usize {
    let mut len = 0;
    let mut in_escape = false;
    for ch in s.chars() {
        if in_escape {
            if ch.is_ascii_alphabetic() {
                in_escape = false;
            }
        } else if ch == '\x1b' {
            in_escape = true;
        } else {
            len += 1;
        }
    }
    len
}

/// Intermediate representation of a single help line item.
/// Built once by the shared assembly logic, then formatted with or without color.
struct HelpEntry {
    /// The left column label (e.g. "-v, --verbose" or "--output <FILE>").
    label: String,
    /// The right column description (e.g. "Enable verbose output (required) [default: x]").
    description: String,
}

// ── Leaf colorization helpers ──────────────────────────────────────────────

#[cfg(feature = "color")]
fn green(s: &str) -> String {
    use nanocolor::Colorize;
    s.green().to_string()
}

#[cfg(not(feature = "color"))]
fn green(s: &str) -> String {
    s.to_string()
}

#[cfg(feature = "color")]
fn cyan(s: &str) -> String {
    use nanocolor::Colorize;
    s.cyan().to_string()
}

#[cfg(not(feature = "color"))]
fn cyan(s: &str) -> String {
    s.to_string()
}

#[cfg(feature = "color")]
fn dim(s: &str) -> String {
    use nanocolor::Colorize;
    s.dim().to_string()
}

#[cfg(not(feature = "color"))]
fn dim(s: &str) -> String {
    s.to_string()
}

#[cfg(feature = "color")]
fn bold_yellow(s: &str) -> String {
    use nanocolor::Colorize;
    s.bold().yellow().to_string()
}

#[cfg(not(feature = "color"))]
fn bold_yellow(s: &str) -> String {
    s.to_string()
}

#[cfg(feature = "color")]
fn bold(s: &str) -> String {
    use nanocolor::Colorize;
    s.bold().to_string()
}

#[cfg(not(feature = "color"))]
fn bold(s: &str) -> String {
    s.to_string()
}

// ── Display width helper ───────────────────────────────────────────────────

/// Returns the visible width of a string.
/// When color is enabled, strips ANSI escape sequences from the count.
/// When color is disabled, returns s.len() directly.
#[cfg(feature = "color")]
fn display_width(s: &str) -> usize {
    visible_len(s)
}

#[cfg(not(feature = "color"))]
fn display_width(s: &str) -> usize {
    s.len()
}

/// Formats a help section with column-aligned descriptions.
/// Uses `bold_yellow()` for the header and `display_width()` for column alignment.
/// Returns empty string if entries is empty.
fn format_section(header: &str, entries: &[HelpEntry]) -> String {
    if entries.is_empty() {
        return String::new();
    }
    let max_width = entries.iter().map(|e| display_width(&e.label)).max().unwrap_or(0);
    let mut out = String::new();
    out.push_str(&format!("\n{}\n", bold_yellow(header)));
    for entry in entries {
        let pad = max_width.saturating_sub(display_width(&entry.label));
        out.push_str(&format!(
            "  {}{:pad$}  {}\n",
            entry.label,
            "",
            entry.description,
            pad = pad
        ));
    }
    out
}

impl ArgParser {
    /// Generate formatted help text for this parser.
    ///
    /// Includes the description, usage line, options, positional arguments,
    /// and subcommands sections. When the `color` feature is enabled and the
    /// terminal supports it, the output includes ANSI color codes.
    pub fn help_text(&self) -> String {
        let mut out = String::new();
        let name = self.program_name.as_deref().unwrap_or("program");

        // Description
        if let Some(ref desc) = self.program_desc {
            out.push_str(desc);
            out.push_str("\n\n");
        }

        // Usage summary line
        let has_subcommands = !self.subcommands.is_empty();
        let has_visible = self.flags.iter().any(|f| !f.hidden) || self.options.iter().any(|o| !o.hidden);

        out.push_str(&format!("{} {}", bold_yellow("Usage:"), bold(name)));
        if has_visible {
            out.push_str(" [OPTIONS]");
        }
        if has_subcommands {
            out.push_str(&format!(" {}", cyan("<SUBCOMMAND>")));
        } else {
            for pos in &self.positionals {
                let suffix = if pos.multi { "..." } else { "" };
                if pos.required {
                    out.push_str(&format!(" {}{}", cyan(&format!("<{}>", pos.name)), suffix));
                } else {
                    out.push_str(&format!(" {}{}", cyan(&format!("[{}]", pos.name)), suffix));
                }
            }
        }
        out.push('\n');

        // Options (flags + options combined under one header)
        {
            let mut entries: Vec<HelpEntry> = Vec::new();
            for flag in self.flags.iter().filter(|f| !f.hidden) {
                let label = match flag.short {
                    Some(c) => format!("{}, {}", green(&format!("-{c}")), green(&format!("--{}", flag.long))),
                    None => format!("    {}", green(&format!("--{}", flag.long))),
                };
                entries.push(HelpEntry {
                    label,
                    description: flag.description.clone(),
                });
            }
            for opt in self.options.iter().filter(|o| !o.hidden) {
                let placeholder_str = cyan(&format!("<{}>", opt.placeholder));
                let label = match opt.short {
                    Some(c) => format!(
                        "{}, {} {}",
                        green(&format!("-{c}")),
                        green(&format!("--{}", opt.long)),
                        placeholder_str
                    ),
                    None => format!("    {} {}", green(&format!("--{}", opt.long)), placeholder_str),
                };
                let req = if opt.required {
                    format!(" {}", dim("(required)"))
                } else {
                    String::new()
                };
                let multi_hint = if opt.multi {
                    format!(" {}", dim("(multiple)"))
                } else {
                    String::new()
                };
                let default_hint = match &opt.default {
                    Some(val) => format!(" {}", dim(&format!("[default: {val}]"))),
                    None => String::new(),
                };
                let env_hint = match &opt.env_var {
                    Some(var) => format!(" {}", dim(&format!("[env: {var}]"))),
                    None => String::new(),
                };
                let validator_hint = opt
                    .validator
                    .as_ref()
                    .and_then(|v| v.hint())
                    .map(|h| format!(" {}", dim(&format!("[{h}]"))))
                    .unwrap_or_default();
                entries.push(HelpEntry {
                    label,
                    description: format!(
                        "{}{req}{multi_hint}{default_hint}{env_hint}{validator_hint}",
                        opt.description
                    ),
                });
            }
            out.push_str(&format_section("Options:", &entries));
        }

        // Positionals (omitted when subcommands are present)
        if !has_subcommands {
            let entries: Vec<HelpEntry> = self
                .positionals
                .iter()
                .map(|pos| {
                    let multi_suffix = if pos.multi { "..." } else { "" };
                    let label = format!("{}{}", green(&pos.name), multi_suffix);
                    let req = if pos.required {
                        format!(" {}", dim("(required)"))
                    } else {
                        String::new()
                    };
                    let default_hint = match &pos.default {
                        Some(val) => format!(" {}", dim(&format!("[default: {val}]"))),
                        None => String::new(),
                    };
                    let validator_hint = pos
                        .validator
                        .as_ref()
                        .and_then(|v| v.hint())
                        .map(|h| format!(" {}", dim(&format!("[{h}]"))))
                        .unwrap_or_default();
                    HelpEntry {
                        label,
                        description: format!("{}{req}{default_hint}{validator_hint}", pos.description),
                    }
                })
                .collect();
            out.push_str(&format_section("Positional arguments:", &entries));
        }

        // Subcommands
        if has_subcommands {
            let entries: Vec<HelpEntry> = self
                .subcommands
                .iter()
                .map(|subcmd| HelpEntry {
                    label: green(&subcmd.name),
                    description: subcmd.description.clone(),
                })
                .collect();
            out.push_str(&format_section("Subcommands:", &entries));
        }

        out
    }
}
