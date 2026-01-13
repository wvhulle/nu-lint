#!/usr/bin/env nu

use std/log

# Build release binary for nu-lint
# Usage: ./build.nu [--cargo] [--cache <name>]

def main [
    --cargo  # Use cargo instead of nix build (for local development)
    --cache: string = "wvhulle"  # Cachix cache name
]: nothing -> nothing {
    mkdir dist
    load-dotenv

    if $cargo {
        build-cargo
    } else {
        build-nix $cache
    }

    create-checksums
    log info "Build complete!"
    print (ls dist)
}

# Load .env file if present (for local development)
def load-dotenv []: nothing -> nothing {
    if not (".env" | path exists) { return }

    log debug "Loading .env file..."
    open .env
        | lines
        | where {|line| $line | str contains "=" }
        | each {|line|
            let idx = $line | str index-of "="
            { ($line | str substring ..<$idx): ($line | str substring ($idx + 1)..) }
        }
        | reduce {|it, acc| $acc | merge $it }
        | load-env $in
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
        ^cachix authtoken (get-cachix-token)
        ^cachix use $cache
        log info "Cachix configured"
    } catch {
        log warning "Cachix setup failed, continuing without cache"
    }
}

# Push build artifacts to Cachix
def push-to-cachix [cache: string]: nothing -> nothing {
    if not (cachix-available) { return }

    log info "Pushing build closure to Cachix..."
    try {
        ^nix-store --query --requisites ./result | ^cachix push $cache
    } catch {
        log warning "Failed to push to cache"
    }
}

# Enable nix flakes in CI environment
def enable-flakes []: nothing -> nothing {
    if not ("/etc/nix/nix.conf" | path exists) { return }
    try { "experimental-features = nix-command flakes\n" | save --append /etc/nix/nix.conf }
}

# Get platform info for binary naming
def get-platform []: nothing -> record<os: string, arch: string> {
    let arch = try { uname | get machine } catch { "x86_64" }
    let os = try { uname | get kernel-name | str downcase } catch { "linux" }

    # Validate to prevent path injection
    for field in [$arch $os] {
        if ($field | str replace --all --regex '[a-zA-Z0-9_-]' '' | is-not-empty) {
            error make { msg: $"Invalid platform component: ($field)" }
        }
    }

    { os: $os, arch: $arch }
}

# Copy binary to dist with platform-specific name
def copy-binary [src: string]: nothing -> string {
    let platform = get-platform
    let target = $"dist/nu-lint-($platform.os)-($platform.arch)"

    ^cp $src $target
    ^chmod +x $target

    $target
}

# Build with Nix
def build-nix [cache: string]: nothing -> nothing {
    log debug "Configuring nix..."
    enable-flakes
    setup-cachix $cache

    log info "Building with nix..."
    ^nix build .#default --print-build-logs

    push-to-cachix $cache

    log debug "Copying binary to dist/..."
    let target = copy-binary "result/bin/nu-lint"
    log info $"Binary ready: ($target)"
}

# Build with Cargo
def build-cargo []: nothing -> nothing {
    log info "Building with cargo..."

    ^cargo build --release

    let target = copy-binary "target/release/nu-lint"
    log info $"Binary ready: ($target)"
}

# Generate SHA256 checksums for dist files
def create-checksums []: nothing -> nothing {
    log debug "Creating checksums..."

    cd dist
    let files = try { ls nu-lint-* | get name } catch { return }

    $files
        | each {|f|
            let hash = open --raw $f | hash sha256
            log debug $"  ($f): ($hash | str substring 0..16)..."
            $"($hash)  ($f)"
        }
        | str join "\n"
        | save -f checksums-sha256.txt
}
