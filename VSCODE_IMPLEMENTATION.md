# VS Code JSON Output Format - Implementation Summary

## Changes Made

### 1. New Output Format: `vscode-json`

Added a new output format specifically designed for VS Code and LSP-compatible editors.

**Usage:**

```bash
nu-lint script.nu --format vscode-json
```

### 2. Key Features

#### LSP-Compatible Structure

- **0-indexed positions**: Lines and characters use 0-based indexing (LSP standard)
- **Numeric severity**: 1=Error, 2=Warning, 3=Information, 4=Hint
- **Standard fields**: `range`, `severity`, `code`, `source`, `message`
- **Diagnostics grouped by file**: Easy consumption by file path

#### Code Actions (Auto-fixes)

Auto-fixes are provided in a structured format ready for VS Code Quick Fixes:

```json
"code_action": {
  "title": "Replace with compound assignment: $x += 10",
  "edits": [
    {
      "range": { "start": {...}, "end": {...} },
      "new_text": "$x += 10"
    }
  ]
}
```

#### Related Information

Additional context follows LSP `DiagnosticRelatedInformation` format:

```json
"related_information": [
  {
    "location": {
      "uri": "file.nu",
      "range": { "start": {...}, "end": {...} }
    },
    "message": "Additional context or help text"
  }
]
```

### 3. Files Modified

- **`src/output.rs`**:
  - Added `format_vscode_json()` function
  - Added VS Code diagnostic structures: `VsCodeDiagnostic`, `VsCodeRange`, `VsCodePosition`, etc.
  - Added `VsCodeJsonOutput` structure
  - Added `lint_level_to_severity()` helper function

- **`src/cli.rs`**:
  - Added `VscodeJson` variant to `Format` enum
  - Updated `output_results()` to handle new format

- **`src/lib.rs`**:
  - Exported new public types and functions

- **`README.md`**:
  - Added output format examples
  - Referenced new documentation

### 4. Documentation

Created comprehensive documentation in `VSCODE_JSON_FORMAT.md`:

- Output structure specification
- Field descriptions
- Example output
- TypeScript integration code for VS Code extensions
- Comparison with standard JSON format

### 5. Output Structure

```json
{
  "diagnostics": {
    "filename.nu": [
      {
        "range": {
          "start": { "line": 0, "character": 0 },
          "end": { "line": 0, "character": 10 }
        },
        "severity": 2,
        "code": "rule_id",
        "source": "nu-lint",
        "message": "Diagnostic message",
        "related_information": [...],  // optional
        "code_action": {...}            // optional
      }
    ]
  },
  "summary": {
    "errors": 0,
    "warnings": 1,
    "info": 0,
    "files_checked": 1
  }
}
```

### 6. Benefits for VS Code Extension Development

1. **Direct mapping to VS Code Diagnostic API**: No complex transformations needed
2. **0-indexed positions**: Matches VS Code's Position API
3. **Grouped by file**: Efficient for multi-file linting
4. **Code actions included**: Quick fixes ready to implement
5. **Related information**: Rich context for users
6. **Streaming-friendly**: Can start parsing as data arrives

### 7. Backward Compatibility

- Original `json` format unchanged
- Original `text` format unchanged  
- New format is opt-in via `--format vscode-json`

## Testing

Tested with:

```bash
# Simple test
cargo run -- benches/fixtures/small_file.nu --format vscode-json

# Complex test with multiple violations
cargo run -- benches/fixtures/with_violations.nu --format vscode-json
```

All tests produce valid JSON with proper LSP structure.

## Next Steps for VS Code Extension

1. Run `nu-lint --format vscode-json` on file save/change
2. Parse JSON output
3. Convert to `vscode.Diagnostic[]` (minimal transformation needed)
4. Apply to `diagnosticCollection`
5. Optionally implement code actions from `code_action` field

See `VSCODE_JSON_FORMAT.md` for complete TypeScript integration example.
