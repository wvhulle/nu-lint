# D002: Exported function docs
# This file should be detected by the linter

# Bad: exported function without docs
export def my-command [] {
    echo "hello world"
}

# Bad: exported function without docs
export def process-data [input: string] {
    $input | str upcase
}

# Good: this is not exported, so no docs required
def helper-function [] {
    echo "helper"
}
