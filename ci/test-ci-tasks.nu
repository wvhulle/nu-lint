#!/usr/bin/env nu

use std/assert
use ./nix-built-paths.nu [nix-config-with-hook store-paths-from-lines]
use ./release-tasks.nu [checksum-line]

def main []: nothing -> nothing {
  assert equal (nix-config-with-hook "max-jobs = 1" "/nix/store/hook") "max-jobs = 1\npost-build-hook = /nix/store/hook"
  assert equal (nix-config-with-hook "" "/nix/store/hook") "\npost-build-hook = /nix/store/hook"
  assert equal ("/nix/store/first\n\n/nix/store/second\n" | store-paths-from-lines) ["/nix/store/first" "/nix/store/second"]
  assert equal ("" | store-paths-from-lines) []
  assert equal (checksum-line "abc123" "nu-lint.tar.gz") "abc123  nu-lint.tar.gz\n"
  print "CI task tests passed"
}
