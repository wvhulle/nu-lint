# P002: Prefer lines over split row "\n"
# This file should be detected by the linter

# Bad: using split row with newline
def get-commits [] {
    ^git log --oneline | split row "\n"
}

# Bad: single quotes with newline
def process-output [] {
    ^ls -la | split row '\n'
}

# Bad: reading file and splitting by newlines
def parse-file [file: string] {
    open $file | split row "\n" | each { |line| echo $line }
}

# Good: this uses a different delimiter, not a newline
def parse-csv [] {
    "a,b,c" | split row ","
}
