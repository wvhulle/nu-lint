[1 2 3 4 5] |
each { |x| $x * 2 } |
where { |x| $x > 4 } |
reduce { |it, acc| $acc + $it } |
$in * 10 | into string
