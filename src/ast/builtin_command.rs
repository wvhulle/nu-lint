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

    fn is_known_builtin_output_command(&self) -> bool;
}

impl CommandExt for str {
    fn is_known_builtin_output_command(&self) -> bool {
        KNOWN_BUILTIN_OUTPUT_COMMANDS.contains(&self)
    }
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
