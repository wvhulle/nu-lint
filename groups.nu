#!/usr/bin/env -S nu

let group_text = (cargo run -- --groups | lines)
let readme_lines = (open ./README.md | lines)

let start = ($readme_lines | enumerate | where item =~ "start-rule-groups" | first | get index)
let end = ($readme_lines | enumerate | where item =~ "end-rule-groups" | first | get index)

let new_readme = [
  ...($readme_lines | take ($start + 1))
  ...($group_text)
  ...($readme_lines | skip $end)
]

$new_readme | str join "\n" | save -f ./README.md
