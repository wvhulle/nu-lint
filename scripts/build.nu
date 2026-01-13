#!/usr/bin/env nu

# Build release binary for nu-lint
# Usage: ./build.nu [--cargo]

def main [
    --cargo  # Use cargo instead of nix build (for local development)
]: nothing -> table {
    try { mkdir dist }

    if $cargo {
        build-cargo
    } else {
        build-nix
    }

    create-checksums
    print $"(ansi green)Build complete!(ansi reset)"
    try { ls dist } catch { [] }
}

def build-nix []: nothing -> nothing {
    print $"(ansi blue)Building with nix...(ansi reset)"

    nix build .#default | complete
    try { cp result/bin/nu-lint dist/nu-lint-linux-x86_64 | complete }
    try { chmod +x dist/nu-lint-linux-x86_64 | complete }
}

def build-cargo []: nothing -> nothing {
    print $"(ansi blue)Building with cargo...(ansi reset)"

    cargo build --release | complete

    let arch = try { uname | get machine } catch { "x86_64" }
    let os = try { uname | get kernel-name | str downcase } catch { "linux" }
    # Validate arch/os only contain safe characters
    if ($arch | str replace --all --regex '[a-zA-Z0-9_-]' '' | str length) > 0 {
        error make {msg: $"Invalid arch: ($arch)"}
    }
    if ($os | str replace --all --regex '[a-zA-Z0-9_-]' '' | str length) > 0 {
        error make {msg: $"Invalid os: ($os)"}
    }
    let target = $"dist/nu-lint-($os)-($arch)"
    try { cp target/release/nu-lint $target | complete }
    try { chmod +x $target | complete }
}

def create-checksums []: nothing -> nothing {
    print $"(ansi blue)Creating checksums...(ansi reset)"

    try { cd dist } catch { return }
    let files = try { ls nu-lint-* | get name } catch { return }

    $files | each { |f|
        let hash = try { open --raw $f | hash sha256 } catch { "error" }
        $"($hash)  ($f)"
    } | str join "\n" | try { save -f checksums-sha256.txt }

    print (try { open checksums-sha256.txt } catch { "No checksums" })
}
