# Nu-Lint

Linter for the innovative [Nu](https://www.nushell.sh/) shell.

Learning to use a new shell is a radical change that can use some assistance. This project is aimed at helping new and intermediate users of the [Nu](https://www.nushell.sh/) shell. Nu shell has a lot of useful features not found in other scripting languages. This linter will give you hints to use all of them and even offer automatic fixes.

All rules are optional and can be disabled with a configuration file. The rule definitions are designed to be compatible with:

- The standard Nu parser  [nu-check](https://www.nushell.sh/commands/docs/nu-check.html).
- The standard Nu formatter [topiary-nushell](https://github.com/blindFS/topiary-nushell).

## Example

The rule `prefer_pipeline_input` recommends to use pipelines instead of positional arguments:

```nu
def filter-positive [numbers] { 
    $numbers | where $it > 0 
}
```

```nu
def filter-positive [] { 
    where $it > 0 
}
```

This rule in particular encourages you to use lazy pipeline input. When you evaluate a traditional positional list argument, the whole list is processed at once, but when you use implicit pipeline input (by starting the function body with `where`), the list processed lazily (without loading the list in memory completely at once).

## Installation

From crates.io:

```bash
cargo install nu-lint
```

### Source

Build from source:

```bash
cargo install --path .
cargo install --git "$THIS_GIT_URL"
```

### Nix

To install in Nix or NixOS, add to `configuration.nix`:

```nix
let
  nu-lint = pkgs.callPackage (pkgs.fetchFromGitHub {
    owner = "wvhulle";
    repo = "nu-lint";
    rev = "COMMIT_HASH";
    sha256 = ""; # nix will tell you the correct hash
  }) {};
in
{
  environment.systemPackages = [
    nu-lint
  ];
}
```

## Editor extension

### VS Code extension

Available at [VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=WillemVanhulle.nu-lint).

### Helix

Add to your `~/.config/helix/languages.toml`:

```toml
[language-server.nu-lint]
command = "nu-lint"
args = ["lsp"]

[[language]]
name = "nu"
language-servers = ["nu-lint"]
```

### Other

You can also implement your own editor extensions using the `lsp` subcommand as in: `nu-lint lsp`. This will spawn a language server compliant with the [Language Server Protocol](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/).

## CLI usage

Run this linter from the command-line with:

```bash
nu-lint                               # Lint working directory
nu-lint script.nu                     # Lint a file or directory
'let x =' | nu-lint                   # Pipe in over stdin
```

Apply automatic fixes:

```bash
nu-lint --fix --dry-run     # Test fixes
nu-lint --fix               # Apply fix
'FRAGMENT' | nu-lint --fix  # Apply fix to STDIN
```

## Configuration

Show all rules:

```bash
nu-lint rules                        
```

Show all rule groups:

```bash
nu-lint groups
```

Create `.nu-lint.toml` in your project root (or any parent directory):

```toml
# Simple format - just list rules and sets with their levels
systemd_journal_prefix = "warn"
snake_case_variables = "deny"
naming = "deny"  # Apply deny level to all rules in the "naming" set

# Or use the structured format for more complex configs
[lints.sets]
performance = "warn"
type-safety = "deny"

[lints.rules]
prefer_pipeline_input = "deny"
max_function_body_length = "allow"
```

Available lint levels: `allow`, `warn`, `deny`.

The linter will automatically find and use this config file when you run it. Otherwise:

```bash
nu-lint --config custom.toml script.nu  
```

## Rules

You can add, remove or change rules by forking this repo and opening a PR (see [./CONTRIBUTING.md](./CONTRIBUTING.md)).

## License

MIT
