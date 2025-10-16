# BP012: Prefer builtin commands over external tools
# This file should be detected by the linter

# Bad: using external ls
def list-directories [] {
    ^ls -la | lines
}

# Bad: using external cat
def read-config [] {
    ^cat config.toml
}

# Bad: using external grep
def search-logs [pattern: string] {
    ^grep $pattern /var/log/app.log
}

# Bad: using external sort
def sort-output [] {
    some-command | ^sort
}

# Bad: using external head
def get-first-lines [] {
    ^head -n 10 file.txt
}

# Good: using built-in commands
def list-dirs-good [] {
    ls | where type == dir
}

# Good: using built-in open
def read-config-good [] {
    open config.toml
}

# Good: custom external tool (no built-in equivalent)
def run-custom [] {
    ^my-custom-tool --flag
}
