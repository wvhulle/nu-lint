# P003: Prefer parse over each with split row for structured text processing

# Bad: Manual splitting with each and split row
^git log --pretty="%h %s" | lines | each { |x| $x | split row " " }

# Bad: Inline split row in each
$lines | each { split row "," }

# Bad: Processing CSV-like data with each and split
$data | lines | each { |line| $line | split row ":" }
