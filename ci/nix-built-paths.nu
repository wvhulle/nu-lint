#!/usr/bin/env nu

const EXPORT_DIRECTORY_NAME = ".nix-built-paths"
const PATH_LIST_FILE_NAME = "paths.txt"
const CACHIX_CACHE_NAME = "wvhulle"
const CACHIX_CACHE_URL = "https://wvhulle.cachix.org"

def export-directory []: nothing -> path {
  $env.CI_WORKSPACE | path join $EXPORT_DIRECTORY_NAME
}

export def store-paths-from-lines []: string -> list<string> {
  lines | compact --empty
}

def exported-store-paths []: nothing -> list<string> {
  let path_list_file = export-directory | path join $PATH_LIST_FILE_NAME
  if ($path_list_file | path exists) {
    open --raw $path_list_file | store-paths-from-lines
  } else {
    []
  }
}

export def nix-config-with-hook [existing_config: string, hook_program: string]: nothing -> string {
  $"($existing_config)\npost-build-hook = ($hook_program)"
}

def "main export-built-paths-hook" []: nothing -> nothing {
  let built_paths = $env.OUT_PATHS | split row " " | compact --empty
  let directory = export-directory
  mkdir $directory
  ^nix copy --quiet --to $"file://($directory)?compression=zstd" ...$built_paths
  $built_paths | each {|built_path| $"($built_path)\n" } | str join | save --append ($directory | path join $PATH_LIST_FILE_NAME)
}

def --wrapped "main build-and-export-paths" [
  ...build_arguments: string
]: nothing -> nothing {
  let hook_config = nix-config-with-hook ($env.NIX_CONFIG? | default "") $env.EXPORT_HOOK_PROGRAM
  with-env {NIX_CONFIG: $hook_config} {
    ^nix build --quiet --print-build-logs ...$build_arguments
  }
}

def "main build-unless-cached" [
  installable: string
]: nothing -> nothing {
  let built_output = ^nix eval --quiet --raw $"($installable).outPath"
  let cache_lookup = do { ^nix path-info --quiet --store $CACHIX_CACHE_URL $built_output } | complete
  if $cache_lookup.exit_code == 0 {
    print $"($built_output) is already in Cachix"
  } else {
    main build-and-export-paths $installable
  }
}

def "main import-exported-paths" []: nothing -> nothing {
  let exported_paths = exported-store-paths
  if ($exported_paths | is-empty) {
    print "No exported paths to import"
  } else {
    ^nix copy --quiet --no-check-sigs --from $"file://(export-directory)" ...$exported_paths
  }
}

def "main upload-exported-paths" []: nothing -> nothing {
  let exported_paths = exported-store-paths
  if ($exported_paths | is-empty) {
    print "No newly built paths to upload"
  } else {
    main import-exported-paths
    ^cachix push $CACHIX_CACHE_NAME ...$exported_paths
  }
}

def main []: nothing -> nothing {
  print "Run a subcommand, for example: build-and-export-paths"
}
