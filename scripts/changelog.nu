#!/usr/bin/env nu

# Generate changelog for release
# Usage: ./changelog.nu [--output <file>]

def main [
  --output (-o): string = "RELEASE_NOTES.md" # Output file path
]: nothing -> string {
  print $"(ansi blue)Generating changelog...(ansi reset)"

  git-cliff --current --strip header -o $output | complete

  # Append installation guide
  let install_guide = generate-install-guide
  $"\n($install_guide)" | save --append $output

  print $"(ansi green)Changelog written to ($output)(ansi reset)"
  print ""
  try { open $output } catch { "" }
}

# Generate installation guide for release notes
def generate-install-guide []: nothing -> string {
  "## Downloads

| File | Platform |
|------|----------|
| `nu-lint-x86_64-unknown-linux-musl.tar.gz` | Linux x86_64 (Ubuntu, Debian, Fedora, etc.) |
| `nu-lint-aarch64-unknown-linux-musl.tar.gz` | Linux ARM64 (Raspberry Pi, AWS Graviton) |

See [README](https://codeberg.org/wvhulle/nu-lint#installation) for full installation instructions.
"
}
