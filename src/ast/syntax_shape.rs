use nu_protocol::SyntaxShape;

/// Extension trait for `SyntaxShape` providing display and conversion utilities
pub trait SyntaxShapeExt {
    /// Convert a `SyntaxShape` to its string representation.
    /// Example: `SyntaxShape::Int.to_type_string()` returns `"int"`
    fn to_type_string(&self) -> String;
}

impl SyntaxShapeExt for SyntaxShape {
    fn to_type_string(&self) -> String {
        match self {
            Self::Int => "int".into(),
            Self::String => "string".into(),
            Self::Float => "float".into(),
            Self::Boolean => "bool".into(),
            Self::List(inner) => format!("list<{}>", inner.to_type_string()),
            Self::Table(cols) if cols.is_empty() => "table".into(),
            Self::Table(cols) => {
                let col_names = cols
                    .iter()
                    .map(|(name, _)| name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("table<{col_names}>")
            }
            Self::Record(_) => "record".into(),
            Self::Filepath => "path".into(),
            Self::Directory => "directory".into(),
            Self::GlobPattern => "glob".into(),
            Self::Filesize => "filesize".into(),
            Self::Duration => "duration".into(),
            Self::DateTime => "datetime".into(),
            Self::Range => "range".into(),
            Self::Number => "number".into(),
            Self::Binary => "binary".into(),
            Self::CellPath => "cell-path".into(),
            Self::Any => "any".into(),
            _ => format!("{self:?}").to_lowercase(),
        }
    }
}
