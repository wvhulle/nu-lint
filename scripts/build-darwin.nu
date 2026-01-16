#!/usr/bin/env nu

# Cross-compile nu-lint for aarch64-apple-darwin using osxcross
# Requires: osxcross environment (runs in darwin-builder Docker container)

def main []: nothing -> nothing {
    let target = "aarch64-apple-darwin"

    # Find macOS SDK
    let sdk = find-macos-sdk
    print $"Using macOS SDK: ($sdk)"

    # Configure cross-compilation environment
    $env.SDKROOT = $sdk
    $env.MACOSX_DEPLOYMENT_TARGET = "13.0"
    $env.CC = "oa64-clang"
    $env.CXX = "oa64-clang++"
    $env.BINDGEN_EXTRA_CLANG_ARGS = $"--sysroot=($sdk) -I($sdk)/usr/include -target ($target)"

    # Build
    rustup target add $target
    cargo build --release --target $target

    # Create tarball
    mkdir dist
    let archive = $"dist/nu-lint-($target).tar.gz"
    tar --create --gzip --file $archive --directory $"target/($target)/release" --transform s,^,nu-lint/, nu-lint

    # Generate checksum
    cd dist
    let hash = open --raw $"nu-lint-($target).tar.gz" | hash sha256
    $"($hash)  nu-lint-($target).tar.gz\n" | save --append checksums-sha256.txt

    print $"Built: ($archive)"
}

def find-macos-sdk []: nothing -> any {
    if ($env.SDKROOT? | is-not-empty) {
        return $env.SDKROOT
    }

    let found = try {
        glob "/usr/local/osxcross/target/SDK/MacOSX*.sdk" | first
    } catch {
        null
    }

    if ($found | is-not-empty) {
        return $found
    }

    error make {msg: "Could not find macOS SDK. Set SDKROOT or ensure osxcross is installed."}
}
