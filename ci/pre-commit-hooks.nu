#!/usr/bin/env nu

def --wrapped "main run-pre-commit-hooks" [...prek_arguments: string]: nothing -> nothing {
  ^prek run --all-files --show-diff-on-failure ...$prek_arguments
}

def main []: nothing -> nothing {
  print "Run a subcommand, for example: run-pre-commit-hooks"
}
