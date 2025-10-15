# BP002: Prefer functional pipelines over mutable list accumulation
mut results = []
for item in [1 2 3] {
    $results = ($results | append $item)
}

mut filtered = []
for x in $data {
    if $x > 10 {
        $filtered = ($filtered | append $x)
    }
}
