# S011: Using pipe to ignore may hide errors
some | pipeline | each { |x| process $x } | ignore
another | operation | ignore
