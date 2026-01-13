#!/usr/bin/env nu

# Generate changelog for release
# Usage: ./changelog.nu [--output <file>]

def main [
    --output (-o): string = "RELEASE_NOTES.md"  # Output file path
]: nothing -> string {
    print $"(ansi blue)Generating changelog...(ansi reset)"

    git-cliff --current --strip header -o $output | complete

    print $"(ansi green)Changelog written to ($output)(ansi reset)"
    print ""
    try { open $output } catch { "" }
}
