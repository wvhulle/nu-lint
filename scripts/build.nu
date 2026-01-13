#!/usr/bin/env nu

# Build release binary for nu-lint
# Usage: ./build.nu [--cargo]

def main [
    --cargo  # Use cargo instead of nix build (for local development)
]: nothing -> nothing {
    try { mkdir dist }

    if $cargo {
        build-cargo
    } else {
        build-nix
    }

    create-checksums
    print $"(ansi green)Build complete!(ansi reset)"
    print (try { ls dist } catch { "No files in dist" })
}

def build-nix []: nothing -> nothing {
    print $"(ansi blue)Building with nix...(ansi reset)"

    # Run nix build and stream output
    let result = do { ^nix build .#default --print-build-logs } | complete
    if $result.exit_code != 0 {
        print $"(ansi red)nix build failed:(ansi reset)"
        print $result.stderr
        error make {msg: "nix build failed"}
    }

    print $"(ansi blue)Copying binary to dist/...(ansi reset)"
    try { ^cp result/bin/nu-lint dist/nu-lint-linux-x86_64 } catch {|e|
        print $"(ansi red)Failed to copy binary: ($e.msg)(ansi reset)"
    }
    try { ^chmod +x dist/nu-lint-linux-x86_64 }

    print $"(ansi green)Binary ready: dist/nu-lint-linux-x86_64(ansi reset)"
}

def build-cargo []: nothing -> nothing {
    print $"(ansi blue)Building with cargo...(ansi reset)"

    let result = do { ^cargo build --release } | complete
    if $result.exit_code != 0 {
        print $"(ansi red)cargo build failed:(ansi reset)"
        print $result.stderr
        error make {msg: "cargo build failed"}
    }

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
    try { ^cp target/release/nu-lint $target }
    try { ^chmod +x $target }

    print $"(ansi green)Binary ready: ($target)(ansi reset)"
}

def create-checksums []: nothing -> nothing {
    print $"(ansi blue)Creating checksums...(ansi reset)"

    try { cd dist } catch { return }
    let files = try { ls nu-lint-* | get name } catch { return }

    $files | each {|f|
        let hash = try { open --raw $f | hash sha256 } catch { "error" }
        print $"  ($f): ($hash | str substring 0..16)..."
        $"($hash)  ($f)"
    } | str join "\n" | try { save -f checksums-sha256.txt }
}
