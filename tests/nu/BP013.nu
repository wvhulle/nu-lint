# BP013: Prefer builtin text transformation commands

# Bad: sed for text replacement
^sed 's/foo/bar/' file.txt

# Bad: awk for processing
^awk '{print $1}' file.txt

# Bad: cut for column selection
^cut -d ',' -f 1 file.csv

# Bad: wc for counting
^wc -l file.txt

# Bad: tr for character translation
^tr 'a-z' 'A-Z' file.txt

# Good: str replace
"hello world" | str replace "world" "universe"

# Good: select for columns
open data.csv | select column1

# Good: length for counting
ls | length

# Good: where for filtering
open data.json | where status == "active"
