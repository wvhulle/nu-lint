# BP010: Use reduce for accumulating values
mut sum = 0
for item in [1 2 3 4 5] {
    $sum = $sum + $item
}

mut product = 1
for num in $numbers {
    $product = $product * $num
}
