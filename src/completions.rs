use std::fmt;
use std::str::FromStr;

use crate::parser::ArgParser;

/// Supported shell targets for completion script generation.
///
/// Use with [`ArgParser::generate_completions`] to produce a shell-specific
/// completion script from your CLI schema. The generated script can be sourced
/// directly or installed into the shell's completion directory.
///
/// `Shell` implements [`FromStr`] (case-insensitive) and [`Display`], so it
/// works naturally as a CLI argument value:
///
/// ```
/// use nanoargs::Shell;
///
/// let shell: Shell = "zsh".parse().unwrap();
/// assert_eq!(shell, Shell::Zsh);
/// assert_eq!(shell.to_string(), "zsh");
/// ```
///
/// Accepted string values: `bash`, `zsh`, `fish`, `powershell` (or `pwsh`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shell {
    /// GNU Bourne-Again Shell.
    Bash,
    /// Z Shell — supports descriptions and mutual-exclusion groups.
    Zsh,
    /// Friendly Interactive Shell.
    Fish,
    /// PowerShell (cross-platform).
    PowerShell,
}

impl FromStr for Shell {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bash" => Ok(Shell::Bash),
            "zsh" => Ok(Shell::Zsh),
            "fish" => Ok(Shell::Fish),
            "powershell" | "pwsh" => Ok(Shell::PowerShell),
            _ => Err(format!("unsupported shell: {s}")),
        }
    }
}

impl fmt::Display for Shell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Shell::Bash => write!(f, "bash"),
            Shell::Zsh => write!(f, "zsh"),
            Shell::Fish => write!(f, "fish"),
            Shell::PowerShell => write!(f, "powershell"),
        }
    }
}

impl ArgParser {
    /// Generate a shell completion script for the given shell.
    ///
    /// Returns a self-contained script string that, when sourced or installed,
    /// provides tab-completion for your CLI's flags, options, and subcommands.
    ///
    /// # Shell-specific behaviour
    ///
    /// | Shell | Descriptions | Short/long exclusion | Script format |
    /// |-------|-------------|---------------------|---------------|
    /// | Bash | No | No | `complete -F` function |
    /// | Zsh | Yes | Yes | `#compdef` + `_arguments` |
    /// | Fish | Yes | No | `complete -c` commands |
    /// | PowerShell | Yes | No | `Register-ArgumentCompleter` |
    ///
    /// # Example
    ///
    /// ```
    /// use nanoargs::{ArgBuilder, Flag, Shell};
    ///
    /// let parser = ArgBuilder::new()
    ///     .name("myapp")
    ///     .flag(Flag::new("verbose").short('v').desc("Enable verbose output"))
    ///     .build()
    ///     .unwrap();
    ///
    /// let script = parser.generate_completions(Shell::Bash);
    /// assert!(script.contains("complete -F"));
    /// ```
    pub fn generate_completions(&self, shell: Shell) -> String {
        match shell {
            Shell::Bash => generate_bash(self),
            Shell::Zsh => generate_zsh(self),
            Shell::Fish => generate_fish(self),
            Shell::PowerShell => generate_powershell(self),
        }
    }
}

fn generate_bash(parser: &ArgParser) -> String {
    let name = parser.program_name().unwrap_or("program");
    let func_name = name.replace('-', "_");
    let mut out = String::new();

    out.push_str(&format!("_{func_name}() {{\n"));
    out.push_str("    local i cur prev opts cmd\n");
    out.push_str("    COMPREPLY=()\n");
    out.push_str("    if type _init_completion &>/dev/null; then\n");
    out.push_str("        _init_completion || return\n");
    out.push_str("    else\n");
    out.push_str("        cur=\"${COMP_WORDS[COMP_CWORD]}\"\n");
    out.push_str("        prev=\"${COMP_WORDS[COMP_CWORD-1]}\"\n");
    out.push_str("    fi\n\n");

    let subs = parser.subcommands();
    if subs.is_empty() {
        // No subcommands — flat completion
        let words = bash_completable_words(parser);
        out.push_str(&format!("    opts=\"{words}\"\n"));
        out.push_str("    COMPREPLY=($(compgen -W \"${opts}\" -- \"${cur}\"))\n");
    } else {
        // With subcommands — track which subcommand we're in via a cmd variable
        out.push_str("    cmd=\"\"\n");
        out.push_str("    for i in \"${COMP_WORDS[@]}\"; do\n");
        out.push_str("        case \"${i}\" in\n");
        for sub in subs {
            out.push_str(&format!("            {}) cmd=\"{}\";;\n", sub.name, sub.name));
        }
        out.push_str("        esac\n");
        out.push_str("    done\n\n");

        out.push_str("    case \"${cmd}\" in\n");
        for sub in subs {
            let sub_words = bash_completable_words(&sub.parser);
            out.push_str(&format!("        {})\n", sub.name));
            out.push_str(&format!("            opts=\"{sub_words}\"\n"));
            out.push_str("            ;;\n");
        }
        // Default: top-level options + subcommand names
        let sub_names: Vec<&str> = subs.iter().map(|s| s.name.as_str()).collect();
        let top_words = bash_completable_words(parser);
        let all_top = format!("{} {}", sub_names.join(" "), top_words);
        out.push_str("        *)\n");
        out.push_str(&format!("            opts=\"{}\"\n", all_top.trim()));
        out.push_str("            ;;\n");
        out.push_str("    esac\n\n");

        out.push_str("    COMPREPLY=($(compgen -W \"${opts}\" -- \"${cur}\"))\n");
    }

    out.push_str("}\n\n");
    out.push_str(&format!("complete -F _{func_name} -o bashdefault -o default {name}\n"));
    out
}

/// Escape single quotes for embedding in bash single-quoted strings.
fn bash_escape(s: &str) -> String {
    s.replace('\'', "'\\''")
}

/// Build the completable word list for a parser (flags + options, non-hidden).
fn bash_completable_words(parser: &ArgParser) -> String {
    let mut words = Vec::new();
    for flag in parser.flags() {
        if flag.hidden {
            continue;
        }
        words.push(format!("--{}", flag.long));
        if let Some(c) = flag.short {
            words.push(format!("-{c}"));
        }
    }
    for opt in parser.options() {
        if opt.hidden {
            continue;
        }
        words.push(format!("--{}", opt.long));
        if let Some(c) = opt.short {
            words.push(format!("-{c}"));
        }
    }
    bash_escape(&words.join(" "))
}

fn generate_zsh(parser: &ArgParser) -> String {
    let name = parser.program_name().unwrap_or("program");
    let func_name = name.replace('-', "_");
    let mut out = String::new();

    out.push_str(&format!("#compdef {name}\n\n"));

    let subs = parser.subcommands();
    if subs.is_empty() {
        // Simple case: no subcommands
        out.push_str(&format!("_{func_name}() {{\n"));
        out.push_str("    _arguments \\\n");
        zsh_push_args(parser, &mut out, "        ");
        out.push_str("}\n\n");
    } else {
        // With subcommands: main function dispatches
        out.push_str(&format!("_{func_name}() {{\n"));
        out.push_str("    local line state\n\n");

        // Top-level flags/options
        out.push_str("    _arguments \\\n");
        zsh_push_args(parser, &mut out, "        ");

        // Subcommand dispatch
        out.push_str("        '*::subcmd:->subcmd' && return\n\n");
        out.push_str("    case $state in\n");
        out.push_str("        subcmd)\n");
        out.push_str("            case $line[1] in\n");
        for sub in subs {
            let sub_func = format!("_{func_name}_{}", sub.name.replace('-', "_"));
            out.push_str(&format!("                {})\n", sub.name));
            out.push_str(&format!("                    {sub_func}\n"));
            out.push_str("                    ;;\n");
        }

        // If no subcommand matched yet, describe available subcommands
        out.push_str("                *)\n");
        out.push_str("                    local -a subcmds\n");
        out.push_str("                    subcmds=(\n");
        for sub in subs {
            let desc = zsh_escape(&sub.description);
            out.push_str(&format!("                        '{}:{}'\n", sub.name, desc));
        }
        out.push_str("                    )\n");
        out.push_str("                    _describe 'subcommand' subcmds\n");
        out.push_str("                    ;;\n");
        out.push_str("            esac\n");
        out.push_str("            ;;\n");
        out.push_str("    esac\n");
        out.push_str("}\n\n");

        // Generate sub-functions for each subcommand
        for sub in subs {
            let sub_func = format!("_{func_name}_{}", sub.name.replace('-', "_"));
            out.push_str(&format!("{sub_func}() {{\n"));
            out.push_str("    _arguments \\\n");
            zsh_push_args(&sub.parser, &mut out, "        ");
            out.push_str("}\n\n");
        }
    }

    // Register the completion function with Zsh's completion system.
    // Works both when sourced directly and when loaded via fpath
    // (the #compdef header handles the fpath case).
    out.push_str(&format!("compdef _{func_name} {name}\n"));
    out
}

/// Escape brackets and colons for Zsh _arguments specs.
fn zsh_escape(s: &str) -> String {
    s.replace('[', "\\[").replace(']', "\\]").replace(':', "\\:")
}

/// Push _arguments specs for all non-hidden flags and options.
fn zsh_push_args(parser: &ArgParser, out: &mut String, indent: &str) {
    for flag in parser.flags() {
        if flag.hidden {
            continue;
        }
        let desc = zsh_escape(&flag.description);
        if let Some(c) = flag.short {
            // Mutual exclusion group: using -r excludes --release and vice versa
            let excl = format!("-{c} --{}", flag.long);
            out.push_str(&format!("{indent}'({excl})--{}[{desc}]' \\\n", flag.long));
            out.push_str(&format!("{indent}'({excl})-{c}[{desc}]' \\\n"));
        } else {
            out.push_str(&format!("{indent}'--{}[{desc}]' \\\n", flag.long));
        }
    }
    for opt in parser.options() {
        if opt.hidden {
            continue;
        }
        let desc = zsh_escape(&opt.description);
        let ph = if opt.placeholder.is_empty() {
            "value".to_string()
        } else {
            opt.placeholder.clone()
        };
        if let Some(c) = opt.short {
            let excl = format!("-{c} --{}", opt.long);
            out.push_str(&format!("{indent}'({excl})--{}[{desc}]:{ph}:' \\\n", opt.long));
            out.push_str(&format!("{indent}'({excl})-{c}[{desc}]:{ph}:' \\\n"));
        } else {
            out.push_str(&format!("{indent}'--{}[{desc}]:{ph}:' \\\n", opt.long));
        }
    }
}

fn generate_fish(parser: &ArgParser) -> String {
    let name = parser.program_name().unwrap_or("program");
    let mut out = String::new();

    let subs = parser.subcommands();
    let has_subs = !subs.is_empty();

    // Top-level flags and options
    for flag in parser.flags() {
        if flag.hidden {
            continue;
        }
        let mut cmd = format!("complete -c {name}");
        if has_subs {
            cmd.push_str(" -n '__fish_use_subcommand'");
        }
        cmd.push_str(&format!(" -l {}", flag.long));
        if let Some(c) = flag.short {
            cmd.push_str(&format!(" -s {c}"));
        }
        if !flag.description.is_empty() {
            cmd.push_str(&format!(" -d '{}'", fish_escape(&flag.description)));
        }
        out.push_str(&cmd);
        out.push('\n');
    }

    for opt in parser.options() {
        if opt.hidden {
            continue;
        }
        let mut cmd = format!("complete -c {name}");
        if has_subs {
            cmd.push_str(" -n '__fish_use_subcommand'");
        }
        cmd.push_str(&format!(" -l {}", opt.long));
        if let Some(c) = opt.short {
            cmd.push_str(&format!(" -s {c}"));
        }
        cmd.push_str(" -r");
        if !opt.description.is_empty() {
            cmd.push_str(&format!(" -d '{}'", fish_escape(&opt.description)));
        }
        out.push_str(&cmd);
        out.push('\n');
    }

    // Subcommands
    for sub in subs {
        // Register the subcommand name itself
        let mut cmd = format!("complete -c {name} -n '__fish_use_subcommand' -a {}", sub.name);
        if !sub.description.is_empty() {
            cmd.push_str(&format!(" -d '{}'", fish_escape(&sub.description)));
        }
        out.push_str(&cmd);
        out.push('\n');

        // Subcommand-specific flags
        for flag in sub.parser.flags() {
            if flag.hidden {
                continue;
            }
            let mut cmd = format!(
                "complete -c {name} -n '__fish_seen_subcommand_from {}' -l {}",
                sub.name, flag.long
            );
            if let Some(c) = flag.short {
                cmd.push_str(&format!(" -s {c}"));
            }
            if !flag.description.is_empty() {
                cmd.push_str(&format!(" -d '{}'", fish_escape(&flag.description)));
            }
            out.push_str(&cmd);
            out.push('\n');
        }

        // Subcommand-specific options
        for opt in sub.parser.options() {
            if opt.hidden {
                continue;
            }
            let mut cmd = format!(
                "complete -c {name} -n '__fish_seen_subcommand_from {}' -l {}",
                sub.name, opt.long
            );
            if let Some(c) = opt.short {
                cmd.push_str(&format!(" -s {c}"));
            }
            cmd.push_str(" -r");
            if !opt.description.is_empty() {
                cmd.push_str(&format!(" -d '{}'", fish_escape(&opt.description)));
            }
            out.push_str(&cmd);
            out.push('\n');
        }
    }

    out
}

/// Escape single quotes for embedding in fish single-quoted strings.
fn fish_escape(s: &str) -> String {
    s.replace('\'', "'\\''")
}

fn generate_powershell(parser: &ArgParser) -> String {
    let name = parser.program_name().unwrap_or("program");
    let mut out = String::new();

    out.push_str(&format!(
        "Register-ArgumentCompleter -CommandName '{name}' -ScriptBlock {{\n"
    ));
    out.push_str("    param($commandName, $wordToComplete, $cursorPosition)\n\n");

    let subs = parser.subcommands();
    if subs.is_empty() {
        // No subcommands — flat completion
        out.push_str("    $completions = @(\n");
        ps_push_completions(parser, &mut out, "        ");
        out.push_str("    )\n\n");
        out.push_str("    $completions | Where-Object { $_.CompletionText -like \"$wordToComplete*\" }\n");
    } else {
        // Find the current subcommand
        out.push_str("    # Determine the subcommand\n");
        out.push_str("    $tokens = $commandName -split '\\s+'\n");
        out.push_str("    $subcmd = $null\n");
        out.push_str("    foreach ($t in $tokens[1..($tokens.Length - 1)]) {\n");
        out.push_str("        switch ($t) {\n");
        for sub in subs {
            out.push_str(&format!("            '{}' {{ $subcmd = '{}' }}\n", sub.name, sub.name));
        }
        out.push_str("        }\n");
        out.push_str("    }\n\n");

        out.push_str("    $completions = @()\n\n");
        out.push_str("    switch ($subcmd) {\n");
        for sub in subs {
            out.push_str(&format!("        '{}' {{\n", sub.name));
            out.push_str("            $completions = @(\n");
            ps_push_completions(&sub.parser, &mut out, "                ");
            out.push_str("            )\n");
            out.push_str("        }\n");
        }
        // Default: top-level completions + subcommand names
        out.push_str("        default {\n");
        out.push_str("            $completions = @(\n");
        ps_push_completions(parser, &mut out, "                ");
        for sub in subs {
            let desc = ps_escape(&sub.description);
            out.push_str(&format!(
                "                [System.Management.Automation.CompletionResult]::new('{}', '{}', 'ParameterValue', '{}')\n",
                sub.name, sub.name, desc
            ));
        }
        out.push_str("            )\n");
        out.push_str("        }\n");
        out.push_str("    }\n\n");

        out.push_str("    $completions | Where-Object { $_.CompletionText -like \"$wordToComplete*\" }\n");
    }

    out.push_str("}\n");
    out
}
/// Escape special PowerShell characters in description strings.
fn ps_escape(s: &str) -> String {
    s.replace('\'', "''")
}

/// Push [CompletionResult]::new() entries for all non-hidden flags and options.
fn ps_push_completions(parser: &ArgParser, out: &mut String, indent: &str) {
    for flag in parser.flags() {
        if flag.hidden {
            continue;
        }
        let desc = ps_escape(&flag.description);
        out.push_str(&format!(
            "{indent}[System.Management.Automation.CompletionResult]::new('--{}', '--{}', 'ParameterName', '{}')\n",
            flag.long, flag.long, desc
        ));
        if let Some(c) = flag.short {
            out.push_str(&format!(
                "{indent}[System.Management.Automation.CompletionResult]::new('-{c}', '-{c}', 'ParameterName', '{}')\n",
                desc
            ));
        }
    }
    for opt in parser.options() {
        if opt.hidden {
            continue;
        }
        let desc = ps_escape(&opt.description);
        out.push_str(&format!(
            "{indent}[System.Management.Automation.CompletionResult]::new('--{}', '--{}', 'ParameterName', '{}')\n",
            opt.long, opt.long, desc
        ));
        if let Some(c) = opt.short {
            out.push_str(&format!(
                "{indent}[System.Management.Automation.CompletionResult]::new('-{c}', '-{c}', 'ParameterName', '{}')\n",
                desc
            ));
        }
    }
}
