# VS Code JSON Output Format

The `vscode-json` output format provides LSP (Language Server Protocol) compatible diagnostics that are easy to parse for VS Code extensions and other editor integrations.

## Usage

```bash
nu-lint <file> --format vscode-json
```

## Output Structure

The output is a JSON object with two main fields:

```json
{
  "diagnostics": {
    "filename.nu": [ /* array of diagnostics */ ]
  },
  "summary": {
    "errors": 1,
    "warnings": 2,
    "info": 0,
    "files_checked": 1
  }
}
```

### Diagnostics

Diagnostics are grouped by file path. Each diagnostic follows the LSP diagnostic format with these fields:

#### Core Fields

- **`range`** (object): The location of the diagnostic
  - **`start`** (object): Start position
    - **`line`** (number): 0-indexed line number
    - **`character`** (number): 0-indexed character position
  - **`end`** (object): End position
    - **`line`** (number): 0-indexed line number
    - **`character`** (number): 0-indexed character position

- **`severity`** (number): Diagnostic severity
  - `1` = Error
  - `2` = Warning
  - `3` = Information
  - `4` = Hint

- **`code`** (string): The rule ID (e.g., `"prefer_compound_assignment"`)

- **`source`** (string): Always `"nu-lint"`

- **`message`** (string): The diagnostic message

#### Optional Fields

- **`related_information`** (array, optional): Additional context for the diagnostic
  - **`location`** (object): Location of related information
    - **`uri`** (string): File path
    - **`range`** (object): Range in the same format as above
  - **`message`** (string): Additional context or help text

- **`code_action`** (object, optional): Available auto-fix
  - **`title`** (string): Description of the fix
  - **`edits`** (array): Text edits to apply
    - **`range`** (object): Range to replace
    - **`new_text`** (string): Replacement text

### Summary

- **`errors`** (number): Count of error-level diagnostics
- **`warnings`** (number): Count of warning-level diagnostics
- **`info`** (number): Count of information-level diagnostics
- **`files_checked`** (number): Number of files analyzed

## Example

```json
{
  "diagnostics": {
    "test.nu": [
      {
        "range": {
          "start": {
            "line": 2,
            "character": 2
          },
          "end": {
            "line": 2,
            "character": 14
          }
        },
        "severity": 2,
        "code": "prefer_compound_assignment",
        "source": "nu-lint",
        "message": "Use compound assignment: $x += instead of $x = $x + ...",
        "related_information": [
          {
            "location": {
              "uri": "test.nu",
              "range": {
                "start": {
                  "line": 2,
                  "character": 2
                },
                "end": {
                  "line": 2,
                  "character": 14
                }
              }
            },
            "message": "Replace with: $x +="
          }
        ],
        "code_action": {
          "title": "Replace with compound assignment: $x += 10",
          "edits": [
            {
              "range": {
                "start": {
                  "line": 2,
                  "character": 2
                },
                "end": {
                  "line": 2,
                  "character": 14
                }
              },
              "new_text": "$x += 10"
            }
          ]
        }
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

## Key Differences from Standard JSON Format

1. **0-indexed positions**: Lines and characters are 0-indexed (LSP standard) instead of 1-indexed
2. **Grouped by file**: Diagnostics are organized in a map keyed by file path
3. **LSP-compatible structure**: Uses standard `range`, `severity`, `code`, `source` fields
4. **Numeric severity**: Uses numbers (1-4) instead of strings ("error", "warning", etc.)
5. **Code actions**: Auto-fixes are provided in a structured format ready for VS Code Quick Fixes
6. **Related information**: Additional context follows LSP `DiagnosticRelatedInformation` format

## Integration with VS Code

To consume this format in a VS Code extension:

```typescript
import * as vscode from 'vscode';
import { exec } from 'child_process';

interface NuLintOutput {
  diagnostics: Record<string, Diagnostic[]>;
  summary: {
    errors: number;
    warnings: number;
    info: number;
    files_checked: number;
  };
}

interface Diagnostic {
  range: {
    start: { line: number; character: number };
    end: { line: number; character: number };
  };
  severity: number;
  code: string;
  source: string;
  message: string;
  related_information?: Array<{
    location: {
      uri: string;
      range: {
        start: { line: number; character: number };
        end: { line: number; character: number };
      };
    };
    message: string;
  }>;
  code_action?: {
    title: string;
    edits: Array<{
      range: {
        start: { line: number; character: number };
        end: { line: number; character: number };
      };
      new_text: string;
    }>;
  };
}

function convertToVsCodeDiagnostic(diag: Diagnostic): vscode.Diagnostic {
  const range = new vscode.Range(
    diag.range.start.line,
    diag.range.start.character,
    diag.range.end.line,
    diag.range.end.character
  );
  
  const severity = [
    undefined,
    vscode.DiagnosticSeverity.Error,
    vscode.DiagnosticSeverity.Warning,
    vscode.DiagnosticSeverity.Information,
    vscode.DiagnosticSeverity.Hint
  ][diag.severity];
  
  const diagnostic = new vscode.Diagnostic(
    range,
    diag.message,
    severity
  );
  
  diagnostic.code = diag.code;
  diagnostic.source = diag.source;
  
  if (diag.related_information) {
    diagnostic.relatedInformation = diag.related_information.map(info => 
      new vscode.DiagnosticRelatedInformation(
        new vscode.Location(
          vscode.Uri.file(info.location.uri),
          new vscode.Range(
            info.location.range.start.line,
            info.location.range.start.character,
            info.location.range.end.line,
            info.location.range.end.character
          )
        ),
        info.message
      )
    );
  }
  
  return diagnostic;
}

async function runNuLint(filePath: string): Promise<vscode.Diagnostic[]> {
  return new Promise((resolve, reject) => {
    exec(`nu-lint ${filePath} --format vscode-json`, (error, stdout, stderr) => {
      if (stderr && !stdout) {
        reject(new Error(stderr));
        return;
      }
      
      try {
        const output: NuLintOutput = JSON.parse(stdout);
        const diagnostics = output.diagnostics[filePath] || [];
        resolve(diagnostics.map(convertToVsCodeDiagnostic));
      } catch (e) {
        reject(e);
      }
    });
  });
}
```

## Notes

- All positions are 0-indexed to match LSP/VS Code expectations
- File paths in `diagnostics` keys match the input file paths
- The `code_action` field provides structured auto-fix information that can be converted to VS Code Quick Fixes
- The format is designed for streaming processing - you can start parsing diagnostics as soon as you receive them
