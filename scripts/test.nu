#!/usr/bin/env nu

# Run tests for nu-lint
# Usage: ./test.nu [--cargo]

def main [
    --cargo  # Use cargo instead of nix build
]: nothing -> nothing {
    print "Running tests..."

    if $cargo {
        print "Running cargo test..."
        ^cargo test --all-targets
    } else {
        print "Running tests with nix-shell..."
        ^nix-shell -p rustc cargo --run "cargo test --all-targets"
    }

    print "Tests complete!"
}
