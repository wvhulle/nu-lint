# BP003: Prefer range iteration over while loops with counters
mut attempts = 0
while $attempts < 10 {
    print $"Attempt ($attempts)"
    $attempts = $attempts + 1
}

mut count = 0
while $count < 5 {
    do_something
    $count += 1
}
