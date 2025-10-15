# P001: Use where for filtering instead of each with if
[1 2 3 4 5] | each { |item| if $item > 2 { $item } }

$data | each { |x| if $x.active { $x } }
