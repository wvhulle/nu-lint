#!/usr/bin/env nu

use std/log

# Build release binaries for nu-lint
# Usage: ./build.nu [--cargo] [--cache <name>] [--targets <list>]

def main [
    --cargo  # Use cargo instead of nix build (for local development)
    --cache: string = "wvhulle"  # Cachix cache name
    --targets: list<string> = []  # Target triples to build (empty = native only)
]: nothing -> nothing {
    mkdir dist
    load-dotenv

    let build_targets = if ($targets | is-empty) {
        [( get-target-triple )]
    } else {
        $targets
    }

    for target in $build_targets {
        log info $"Building for ($target)..."
        if $cargo {
            build-cargo $target
        } else {
            build-nix $cache $target
        }
    }

    create-checksums
    log info "Build complete!"
    print (ls dist)
}

# Load .env file if present (for local development)
def --env load-dotenv []: nothing -> nothing {
    if not (".env" | path exists) { return }

    log debug "Loading .env file..."
    open .env
        | lines
        | where {|line| ($line | str trim | str starts-with "#" | not $in) and ($line | str contains "=") }
        | each {|line|
            let idx = $line | str index-of "="
            { ($line | str substring ..<$idx): ($line | str substring ($idx + 1)..) }
        }
        | reduce --fold {} {|it, acc| $acc | merge $it }
        | load-env
}

# Get Cachix token from environment (CI or local .env)
def get-cachix-token []: nothing -> string {
    $env.CACHIX_AUTH_TOKEN? | default ($env.CACHIX_TOKEN? | default "")
}

def cachix-available []: nothing -> bool {
    (which cachix | length) > 0 and (get-cachix-token | is-not-empty)
}

# Configure Cachix for pulling cached artifacts
def setup-cachix [cache: string]: nothing -> nothing {
    if not (cachix-available) {
        log warning "Cachix not available or no token, building without cache"
        return
    }

    log info $"Setting up Cachix cache: ($cache)"
    try {
        cachix authtoken (get-cachix-token)
        cachix use $cache
        log info "Cachix configured"
    } catch {
        log warning "Cachix setup failed, continuing without cache"
    }
}

# Push build artifacts to Cachix
# With crane, deps are built separately and cached automatically
def push-to-cachix [cache: string, result_link: string]: nothing -> nothing {
    if not (cachix-available) { return }

    log info "Pushing to Cachix..."
    try {
        # Push the result and its runtime closure
        let paths = nix-store --query --requisites $result_link | lines
        if ($paths | is-not-empty) {
            log info $"Pushing ($paths | length) paths to Cachix..."
            cachix push $cache $paths
            log info "Push complete"
        }
    } catch {|e|
        log warning $"Failed to push to cache: ($e.msg)"
    }
}

# Push dependency artifacts to Cachix (crane's buildDepsOnly output)
def push-deps-to-cachix [cache: string]: nothing -> nothing {
    if not (cachix-available) { return }

    log info "Building and pushing deps to Cachix..."
    try {
        let deps_link = "result-deps"
        ^nix build ".#deps" --out-link $deps_link --print-build-logs
        let paths = ^nix-store --query --requisites $deps_link | lines
        if ($paths | is-not-empty) {
            log info $"Pushing ($paths | length) dep paths to Cachix..."
            ^cachix push $cache ...$paths
            log info "Deps push complete"
        }
        rm $deps_link
    } catch {|e|
        log warning $"Failed to push deps to cache: ($e.msg)"
    }
}

# Enable nix flakes in CI environment
def enable-flakes []: nothing -> nothing {
    if not ("/etc/nix/nix.conf" | path exists) { return }
    try { "experimental-features = nix-command flakes\n" | save --append /etc/nix/nix.conf }
}

# Get Rust target triple for binstall compatibility
def get-target-triple []: nothing -> string {
    let arch = try { uname | get machine } catch { "x86_64" }
    let os = try { uname | get kernel-name | str downcase } catch { "linux" }

    # Validate to prevent path injection
    for field in [$arch $os] {
        if ($field | str replace --all --regex '[a-zA-Z0-9_-]' '' | is-not-empty) {
            error make { msg: $"Invalid platform component: ($field)" }
        }
    }

    # Map to Rust target triples (binstall expects these)
    let target = match [$os $arch] {
        ["linux" "x86_64"] => "x86_64-unknown-linux-gnu"
        ["linux" "aarch64"] => "aarch64-unknown-linux-gnu"
        ["darwin" "x86_64"] => "x86_64-apple-darwin"
        ["darwin" "arm64"] => "aarch64-apple-darwin"
        ["darwin" "aarch64"] => "aarch64-apple-darwin"
        _ => $"($arch)-unknown-($os)-gnu"
    }

    $target
}

# Copy binary to dist as tarball with binstall-compatible name
def copy-binary [src: string, target_triple: string]: nothing -> string {
    let archive_name = $"nu-lint-($target_triple).tar.gz"
    let archive_path = $"dist/($archive_name)"

    # Create tarball with binary inside (binstall expects this)
    log debug $"Creating archive ($archive_name)..."
    ^tar -czf $archive_path -C ($src | path dirname) ($src | path basename)

    $archive_path
}

# Build with Nix for a specific target
def build-nix [cache: string, target_triple: string]: nothing -> nothing {
    log debug "Configuring nix..."
    enable-flakes
    setup-cachix $cache

    # Map target triple to nix package attribute
    let nix_attr = match $target_triple {
        "x86_64-unknown-linux-gnu" => "default"
        "aarch64-unknown-linux-gnu" => "aarch64-linux"
        "x86_64-apple-darwin" => "default"
        "aarch64-apple-darwin" => "default"
        _ => {
            log warning $"Unknown target ($target_triple), using default"
            "default"
        }
    }

    # Use unique result symlink per target to avoid conflicts
    let result_link = $"result-($target_triple)"

    log info $"Building with nix \(($nix_attr)\) for ($target_triple)..."
    ^nix build $".#($nix_attr)" --out-link $result_link --print-build-logs

    # Push deps first (crane's buildDepsOnly), then the package
    push-deps-to-cachix $cache
    push-to-cachix $cache $result_link

    log debug "Copying binary to dist/..."
    let archive = copy-binary $"($result_link)/bin/nu-lint" $target_triple
    log info $"Binary ready: ($archive)"

    rm $result_link
}

# Build with Cargo for a specific target
def build-cargo [target_triple: string]: nothing -> nothing {
    log info $"Building with cargo for ($target_triple)..."

    let native = get-target-triple
    if $target_triple == $native {
        ^cargo build --release
        let archive = copy-binary "target/release/nu-lint" $target_triple
        log info $"Binary ready: ($archive)"
    } else {
        # Cross-compilation requires the target to be installed
        ^rustup target add $target_triple
        ^cargo build --release --target $target_triple
        let archive = copy-binary $"target/($target_triple)/release/nu-lint" $target_triple
        log info $"Binary ready: ($archive)"
    }
}

# Generate SHA256 checksums for dist files
def create-checksums []: nothing -> nothing {
    log debug "Creating checksums..."

    cd dist
    let files = try { ls *.tar.gz | get name } catch { return }

    $files
        | each {|f|
            let hash = open --raw $f | hash sha256
            log debug $"  ($f): ($hash | str substring 0..16)..."
            $"($hash)  ($f)"
        }
        | str join "\n"
        | save -f checksums-sha256.txt
}
