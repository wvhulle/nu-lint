#!/usr/bin/env nu

# Build release binary for nu-lint
# Usage: ./build.nu [--cargo]

def main [
    --cargo  # Use cargo instead of nix build (for local development)
]: nothing -> table {
    mkdir dist

    if $cargo {
        build-cargo
    } else {
        build-nix
    }

    create-checksums
    print $"(ansi green)Build complete!(ansi reset)"
    ls dist
}

def build-nix []: nothing -> nothing {
    print $"(ansi blue)Building with nix...(ansi reset)"

    ^nix build .#default | complete | ignore
    ^cp result/bin/nu-lint dist/nu-lint-linux-x86_64 | complete | ignore
    ^chmod +x dist/nu-lint-linux-x86_64 | complete | ignore
}

def build-cargo []: nothing -> nothing {
    print $"(ansi blue)Building with cargo...(ansi reset)"

    ^cargo build --release | complete | ignore

    let arch = uname | get machine
    let os = uname | get kernel-name | str downcase
    ^cp target/release/nu-lint $"dist/nu-lint-($os)-($arch)" | complete | ignore
    ^chmod +x $"dist/nu-lint-($os)-($arch)" | complete | ignore
}

def create-checksums []: nothing -> nothing {
    print $"(ansi blue)Creating checksums...(ansi reset)"

    cd dist
    let files = ls nu-lint-* | get name

    $files | each { |f|
        let hash = open --raw $f | hash sha256
        $"($hash)  ($f)"
    } | str join "\n" | save -f checksums-sha256.txt

    print (open checksums-sha256.txt)
}
