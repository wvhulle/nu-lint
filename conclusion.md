Based on these real-world examples, each should be suggested when:

1. Simple transformation: Each input item is transformed to create an output item

    ```nu
    # BAD
    mut output = []
    for x in $input {
        $output = ($output | append ($x * 2))
    }

    # GOOD  
    $input | each { |x| $x * 2 }
    ```

2. Filtering with transformation: Items are conditionally added with some transformation

    ```nu
    # BAD
    mut results = []
    for x in $input {
        if $x > 5 {
            $results = ($results | append ($x * 2))
        }
    }

    # GOOD
    $input | where $it > 5 | each { |x| $x * 2 }
    # or
    $input | each { |x| if $x > 5 { $x * 2 } }
    ```

where should be suggested when: Simple filtering without transformation:

```nu
# BAD
mut filtered = []
for x in $input {
    if $x > 5 {
        $filtered = ($filtered | append $x)
    }
}

# GOOD
$input | where $it > 5
```

Direct use should be suggested when: Just copying items unchanged

```nu
# BAD
mut data = []
for x in [1 2 3] {
    $data = ($data | append $x)
}

# GOOD
let data = [1 2 3]
```

Should NOT be flagged:

- Pagination loops (like the gh-completions example)
- Complex categorization/grouping into multiple lists
- Loops with early breaks/continues based on complex state
- Accumulation with side effects (API calls, file writes, etc.)

So the answer to your question: Use each when there's a transformation applied to each item (like $x * 2, $x | somecommand, accessing fields, etc.).
