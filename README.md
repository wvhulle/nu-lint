# Nu-Lint

Linter for the innovative [Nu](https://www.nushell.sh/) shell.

Learning to use a new shell is a radical change that can use some assistance. This project is aimed at helping new users of the [Nu](https://www.nushell.sh/) shell. Nu shell has a lot of interesting and useful features and this program will give you hints to use all the features of Nu.

For example, the rule `prefer_pipeline_input` in this program recommends to use pipelines instead of positional arguments (to use lazy instead of eager list processing):

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

All rules are optional and can be disabled with a configuration file. The rule definitions are designed to be compatible with:

- The standard Nu parser  [nu-check](https://www.nushell.sh/commands/docs/nu-check.html).
- The standard Nu formatter [topiary-nushell](https://github.com/blindFS/topiary-nushell).

## Installation

### Cross-platform

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

### VS Code extension

Available at [VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=WillemVanhulle.nu-lint).

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

## Usage

Basic:

```bash
nu-lint                               # Lint working directory
nu-lint script.nu                     # Lint a file
nu-lint directory/                    # Lint directory
'let x =' | nu-lint                   # Pipe in over stdin
```

Apply fixes (also works with STDIN):

```bash
nu-lint --fix --dry-run   # Test
nu-lint --fix             # Apply
```

Output formats:

```bash
nu-lint script.nu --format text            # Human-readable (default)
nu-lint script.nu --format lsp             # JSON 
```

## Configuration

Show all rules:

```bash
nu-lint list-rules                         
```

Show all rule sets:

```bash
nu-lint list-sets
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
