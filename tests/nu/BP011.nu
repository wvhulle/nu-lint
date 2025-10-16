# BP011: Descriptive error messages
# This file should be detected by the linter

# Bad: generic error message
def process-file [file: string] {
    if not ($file | path exists) {
        error make { msg: "error" }
    }
}

# Bad: vague "failed" message
def convert-data [input] {
    if ($input | is-empty) {
        error make { msg: "failed" }
    }
}

# Bad: generic "something went wrong"
def validate [data] {
    error make { msg: "something went wrong" }
}
