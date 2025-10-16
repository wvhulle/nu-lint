# BP012: Prefer builtin commands for common file and text operations
# Based on https://www.nushell.sh/book/coming_from_bash.html#command-equivalents

# Bad: ls -> use builtin ls
^ls -la

# Bad: cat <path> -> use open --raw <path>
^cat config.toml

# Bad: grep <pattern> -> use where $it =~ <substring> or find <substring>
^grep "error" logs.txt

# Bad: head -5 -> use first 5
^head -n 5 file.txt

# Bad: tail -10 -> use last 10
^tail -n 10 file.txt

# Bad: find . -name *.rs -> use ls **/*.rs
^find . -name "*.rs"

# Bad: sort -> use sort or sort-by
^sort file.txt

# Bad: uniq -> use uniq or uniq-by
^uniq file.txt

# Good: using built-in commands
ls | where type == dir

# Good: using built-in open for structured data
open config.toml

# Good: using where for filtering
ls | where size > 1mb

# Good: using first/last
ls | first 5

# Good: using sort
ls | sort-by size

# Good: custom external tool (no built-in equivalent)
^my-custom-tool --flag

# Good: git commands (no built-in equivalent)
^git status
