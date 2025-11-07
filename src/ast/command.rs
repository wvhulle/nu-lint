/// Constants for known external commands that produce output
pub const KNOWN_EXTERNAL_OUTPUT_COMMANDS: &[&str] = &[
    "echo", "ls", "cat", "find", "grep", "curl", "wget", "head", "tail", "sort",
    "whoami",   // Prints the effective username
    "hostname", // Prints the system's hostname
    "pwd",      // Prints the present working directory
    "tty",      // Prints the file name of the terminal connected to stdin
    "id",       // Prints real and effective user and group IDs
    "who",      // Prints who is logged on (always lists at least the current session)
    "date",     // Prints the current date and time
    "uptime",   // Prints system uptime and load
    "uname",    // Prints system information (e.g., "Linux", "Darwin")
    "df",       // Prints filesystem disk space usage (always lists mounts)
    "ps",       // Prints process status (at least lists itself and the parent shell)
    "history",  // Prints the command history
];

/// Constants for known external commands that don't produce output
pub const KNOWN_EXTERNAL_NO_OUTPUT_COMMANDS: &[&str] = &[
    "cd", "mkdir", "rm", "mv", "cp", "touch", "exit", "clear", "ln",    // Creates a link
    "chmod", // Changes file permissions
    "chown", // Changes file ownership
    "chgrp", // Changes file group
    "kill",  // Sends a signal to a process (like SIGTERM)
    "sleep", // Pauses execution for a set time
];

/// Commands that are known to produce output but may have `Type::Any` in their
/// signature
pub const KNOWN_BUILTIN_OUTPUT_COMMANDS: &[&str] = &[
    "ls",
    "http get",
    "http post",
    "open",
    "from json",
    "from csv",
    "select",
    "where",
    "get",
    "find",
    "each",
    "reduce",
    "sort-by",
    "group-by",
    "echo",
];

/// Extension trait for Nushell builtin commands
pub trait CommandExt {
    fn input_type(&self) -> Option<&'static str>;
    fn output_type(&self) -> Option<&'static str>;
    fn is_side_effect_only(&self) -> bool;
}

impl CommandExt for str {
    fn input_type(&self) -> Option<&'static str> {
        match self {
            "each" | "where" | "filter" | "reduce" | "map" | "length" => Some("list<any>"),
            "lines" | "split row" => Some("string"),
            _ => None,
        }
    }

    fn output_type(&self) -> Option<&'static str> {
        match self {
            "str trim" | "str replace" | "str upcase" | "str downcase" | "str contains" => {
                Some("string")
            }
            "each" | "where" | "filter" | "map" => Some("list<any>"),
            "reduce" | "append" | "prepend" => Some("list"),
            "length" => Some("int"),
            "to json" => Some("string"),
            "lines" => Some("list<string>"),
            "is-empty" => Some("bool"),
            _ => None,
        }
    }

    fn is_side_effect_only(&self) -> bool {
        matches!(
            self,
            "print"
                | "println"
                | "eprintln"
                | "error"
                | "mkdir"
                | "rm"
                | "cp"
                | "mv"
                | "touch"
                | "cd"
                | "hide"
                | "use"
                | "overlay"
                | "export"
                | "def"
                | "alias"
                | "module"
                | "const"
                | "let"
                | "mut"
                | "source"
                | "source-env"
        )
    }
}

/// Extension trait for checking external command categories
pub trait ExternalCommandExt {
    fn is_known_external_output_command(&self) -> bool;
    fn is_known_external_no_output_command(&self) -> bool;
    fn is_known_builtin_output_command(&self) -> bool;
}

impl ExternalCommandExt for str {
    fn is_known_external_output_command(&self) -> bool {
        KNOWN_EXTERNAL_OUTPUT_COMMANDS.contains(&self)
    }

    fn is_known_external_no_output_command(&self) -> bool {
        KNOWN_EXTERNAL_NO_OUTPUT_COMMANDS.contains(&self)
    }

    fn is_known_builtin_output_command(&self) -> bool {
        KNOWN_BUILTIN_OUTPUT_COMMANDS.contains(&self)
    }
}
