use nu_protocol::ast::ExternalArgument;

use crate::{
    Violation,
    alternatives::{detect_external_commands, external_args_slices},
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement},
};

const NOTE_SIMPLE: &str =
    "Use the Nushell built-in which provides structured output and better integration.";
const NOTE_ENV: &str = "Use '$env' to access environment variables or 'env' to view all.";
const NOTE_UNAME: &str = "Use 'sys host' to get system information.";
const NOTE_TEE: &str = "Use 'tee { save file.txt }' to save while passing through.";
const NOTE_TR: &str = "Use 'str replace' or case conversion commands.";
const NOTE_REV: &str = "Use 'str reverse' for strings or 'reverse' for lists.";

fn build_fix(
    cmd_text: &str,
    builtin_cmd: &str,
    args: &[ExternalArgument],
    expr_span: nu_protocol::Span,
    context: &LintContext,
) -> Fix {
    let args_text: Vec<&str> = external_args_slices(args, context).collect();

    // Build replacement based on command
    let (new_text, description) = match cmd_text {
        // Simple replacements
        "whoami" | "clear" | "exit" | "stat" | "pwd" | "mkdir" | "rm" | "mv" | "cp" | "touch"
        | "sleep" | "kill" => build_simple_replacement(cmd_text, &args_text, builtin_cmd),
        "cd" => build_cd_replacement(&args_text),
        "env" | "printenv" => build_env_replacement(&args_text),
        "date" => build_date_replacement(),
        "hostname" => build_hostname_replacement(),
        "uname" => build_uname_replacement(),
        "man" => build_man_replacement(&args_text),
        "which" | "type" => build_which_replacement(&args_text),
        "read" => build_read_replacement(&args_text),
        "echo" | "printf" => build_print_replacement(&args_text),
        // Text transformation commands
        "awk" => build_awk_replacement(),
        "cut" => build_cut_replacement(),
        "wc" => build_wc_replacement(&args_text),
        "tr" => build_tr_replacement(),
        "tee" => build_tee_replacement(&args_text),
        "rev" => build_rev_replacement(),
        _ => (
            builtin_cmd.to_string(),
            format!("Use Nu's built-in '{builtin_cmd}'"),
        ),
    };

    Fix {
        explanation: description.into(),
        replacements: vec![Replacement {
            span: expr_span,
            replacement_text: new_text.into(),
        }],
    }
}

fn build_simple_replacement(
    cmd_text: &str,
    args_text: &[&str],
    _builtin: &str,
) -> (String, String) {
    let repl = if args_text.is_empty() {
        cmd_text.to_string()
    } else {
        format!("{} {}", cmd_text, args_text.join(" "))
    };
    (repl, format!("Use Nu's built-in '{cmd_text}'"))
}

fn build_cd_replacement(args_text: &[&str]) -> (String, String) {
    let repl = if args_text.is_empty() {
        "cd".to_string()
    } else {
        format!("cd {}", args_text[0])
    };
    (repl, "Use Nu's built-in 'cd'".to_string())
}

fn build_env_replacement(args_text: &[&str]) -> (String, String) {
    if args_text.is_empty() {
        (
            "$env".to_string(),
            "Use '$env' to access all environment variables as a record".to_string(),
        )
    } else {
        (
            format!("$env.{}", args_text[0]),
            format!(
                "Use '$env.{}' to access environment variable directly",
                args_text[0]
            ),
        )
    }
}

fn build_date_replacement() -> (String, String) {
    (
        "date now".to_string(),
        "Use 'date now' which returns a datetime object with timezone support".to_string(),
    )
}

fn build_hostname_replacement() -> (String, String) {
    (
        "(sys host).hostname".to_string(),
        "Use '(sys host).hostname' for hostname, or 'sys net | get ip' for IP addresses"
            .to_string(),
    )
}

fn build_uname_replacement() -> (String, String) {
    (
        "sys host".to_string(),
        "Use 'sys host' which returns structured system information (name, kernel_version, \
         os_version, etc.)"
            .to_string(),
    )
}

fn build_man_replacement(args_text: &[&str]) -> (String, String) {
    let repl = args_text
        .first()
        .map_or_else(|| "help commands".to_string(), |cmd| format!("help {cmd}"));
    (
        repl,
        "Use 'help <command>' for command help, or 'help commands' to list all commands"
            .to_string(),
    )
}

fn build_which_replacement(args_text: &[&str]) -> (String, String) {
    let repl = args_text
        .first()
        .map_or_else(|| "which".to_string(), |cmd| format!("which {cmd}"));
    (
        repl,
        "Use Nu's built-in 'which' to find command locations".to_string(),
    )
}

fn build_read_replacement(args_text: &[&str]) -> (String, String) {
    if args_text.iter().any(|&s| s == "-s" || s == "--silent") {
        (
            "input -s".to_string(),
            "Use 'input -s' for secure password input (hidden)".to_string(),
        )
    } else {
        (
            "input".to_string(),
            "Use 'input' to read user input".to_string(),
        )
    }
}

fn build_print_replacement(args_text: &[&str]) -> (String, String) {
    (
        if args_text.is_empty() {
            "print".to_string()
        } else {
            format!("print {}", args_text.join(" "))
        },
        "Use 'print' for output (supports structured data)".to_string(),
    )
}

fn build_awk_replacement() -> (String, String) {
    let desc = "Use Nu's data pipeline: 'where' for filtering, 'select' for columns, or 'each' \
                for row processing"
        .to_string();
    ("where | select | each".to_string(), desc)
}

fn build_cut_replacement() -> (String, String) {
    (
        "select".to_string(),
        "Use 'select' to choose columns from structured data".to_string(),
    )
}

fn build_wc_replacement(args_text: &[&str]) -> (String, String) {
    if args_text.contains(&"-l") {
        (
            "lines | length".to_string(),
            "Use 'lines | length' to count lines in a file".to_string(),
        )
    } else {
        (
            "length".to_string(),
            "Use 'length' for item count or 'str length' for character count".to_string(),
        )
    }
}

fn build_tr_replacement() -> (String, String) {
    (
        "str replace".to_string(),
        "Use 'str replace' for character replacement, or 'str upcase'/'str downcase' for case \
         conversion"
            .to_string(),
    )
}

fn build_tee_replacement(args_text: &[&str]) -> (String, String) {
    let repl = args_text.first().map_or_else(
        || "tee { save ... }".to_string(),
        |file| format!("tee {{ save {file} }}"),
    );
    let desc = "Use 'tee { save file.txt }' to save data while passing it through the pipeline"
        .to_string();
    (repl, desc)
}

fn build_rev_replacement() -> (String, String) {
    (
        "str reverse".to_string(),
        "Use 'str reverse' for string reversal or 'reverse' for list reversal".to_string(),
    )
}

#[allow(clippy::too_many_lines, reason = "Legacy grouped rule; remaining commands pending split.")]
fn check(context: &LintContext) -> Vec<Violation> {
    let mut v = Vec::new();
    // System info
    v.extend(detect_external_commands(
        context,
        "prefer_builtin_other",
        "env",
        "$env",
        NOTE_ENV,
        Some(build_fix),
    ));
    // split into dedicated module: printenv
    // split into dedicated module: date
    v.extend(detect_external_commands(
        context,
        "prefer_builtin_other",
        "whoami",
        "whoami",
        NOTE_SIMPLE,
        Some(build_fix),
    ));
    // split into dedicated module: hostname
    v.extend(detect_external_commands(
        context,
        "prefer_builtin_other",
        "uname",
        "sys host",
        NOTE_UNAME,
        Some(build_fix),
    ));
    v.extend(detect_external_commands(
        context,
        "prefer_builtin_other",
        "stat",
        "stat",
        NOTE_SIMPLE,
        Some(build_fix),
    ));
    // Process/system control
    v.extend(detect_external_commands(
        context,
        "prefer_builtin_other",
        "sleep",
        "sleep",
        NOTE_SIMPLE,
        Some(build_fix),
    ));
    v.extend(detect_external_commands(
        context,
        "prefer_builtin_other",
        "kill",
        "kill",
        NOTE_SIMPLE,
        Some(build_fix),
    ));
    v.extend(detect_external_commands(
        context,
        "prefer_builtin_other",
        "clear",
        "clear",
        NOTE_SIMPLE,
        Some(build_fix),
    ));
    v.extend(detect_external_commands(
        context,
        "prefer_builtin_other",
        "exit",
        "exit",
        NOTE_SIMPLE,
        Some(build_fix),
    ));
    // split into dedicated module: man
    // split into dedicated module: which/type
    // split into dedicated module: read
    // File operations
    v.extend(detect_external_commands(
        context,
        "prefer_builtin_other",
        "pwd",
        "pwd",
        NOTE_SIMPLE,
        Some(build_fix),
    ));
    v.extend(detect_external_commands(
        context,
        "prefer_builtin_other",
        "cd",
        "cd",
        NOTE_SIMPLE,
        Some(build_fix),
    ));
    v.extend(detect_external_commands(
        context,
        "prefer_builtin_other",
        "mkdir",
        "mkdir",
        NOTE_SIMPLE,
        Some(build_fix),
    ));
    v.extend(detect_external_commands(
        context,
        "prefer_builtin_other",
        "rm",
        "rm",
        NOTE_SIMPLE,
        Some(build_fix),
    ));
    v.extend(detect_external_commands(
        context,
        "prefer_builtin_other",
        "mv",
        "mv",
        NOTE_SIMPLE,
        Some(build_fix),
    ));
    v.extend(detect_external_commands(
        context,
        "prefer_builtin_other",
        "cp",
        "cp",
        NOTE_SIMPLE,
        Some(build_fix),
    ));
    v.extend(detect_external_commands(
        context,
        "prefer_builtin_other",
        "touch",
        "touch",
        NOTE_SIMPLE,
        Some(build_fix),
    ));
    // split into dedicated module: echo/printf
    // Text processing
    // split into dedicated modules: awk, cut, wc
    v.extend(detect_external_commands(
        context,
        "prefer_builtin_other",
        "tee",
        "tee",
        NOTE_TEE,
        Some(build_fix),
    ));
    v.extend(detect_external_commands(
        context,
        "prefer_builtin_other",
        "tr",
        "str replace",
        NOTE_TR,
        Some(build_fix),
    ));
    v.extend(detect_external_commands(
        context,
        "prefer_builtin_other",
        "rev",
        "str reverse or reverse",
        NOTE_REV,
        Some(build_fix),
    ));
    v
}

pub const fn rule() -> Rule {
    Rule::new(
        "prefer_builtin_other",
        "Avoid external commands when Nushell built-ins are available (env, date, whoami, man, \
         sed, awk, cut, wc, tr, tee, etc.)",
        check,
    )
}

#[cfg(test)]
mod tests;
