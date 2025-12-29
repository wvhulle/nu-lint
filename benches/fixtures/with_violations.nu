# File with intentional lint violations for benchmarking
# This file should parse correctly but trigger multiple lint rules

# replace_by_builtin: Use builtin commands instead of external commands
def list-files [] {
  ^ls | lines
  ^cat /etc/hosts | lines
  ^grep "pattern" file.txt | lines
  ^head -n 10 file.txt | lines
  ^tail -n 5 file.txt | lines
  ^find . -name "*.nu" | lines
  ^sort file.txt | lines
  ^uniq file.txt | lines
}

# prefer_compound_assignment: Use += instead of = $x + 1
def increment-counter [] {
  mut counter = 0
  $counter = $counter + 1
  $counter = $counter - 1
  $counter = $counter * 2
  $counter
}

# unnecessary_mut: Variable doesn't need to be mutable
def unused-mut [] {
  mut x = 5
  $x
}

# unnecessary_variable_before_return: Return expression directly
def redundant-var [] {
  let result = 10 + 20
  $result
}

# prefer_is_not_empty: Use is-empty instead of length comparison
def check-empty [list] {
  if ($list | length) > 0 {
    print "not empty"
  }
  if ($list | length) == 0 {
    print "empty"
  }
}

# prefer_where_over_each_if: Use where instead of each with if
def filter-items [items] {
  $items | each {|item|
    if $item > 10 {
      $item
    } else {
      null
    }
  } | compact
}

# prefer_pipeline_input: Pass data through pipeline
def process-data [data] {
  each {|x| $x * 2 } $data
}

# collapsible_if: Nested if statements can be combined
def nested-conditions [x y] {
  if $x > 0 {
    if $y > 0 {
      print "both positive"
    }
  }
}

# prefer_range_iteration: Use range instead of loop
def count-to-ten [] {
  mut i = 0
  loop {
    if $i >= 10 {
      break
    }
    print $i
    $i = $i + 1
  }
}

# prefer_lines_over_split: Use lines instead of split row
def split-text [text] {
  $text | split row "\n"
}

# missing_type_annotation: Functions should have type annotations
def no-types [x y] {
  $x + $y
}

# exported_function_docs: Exported functions should have documentation
export def undocumented [] {
  print "no docs"
}

# prefer_parse_command: Use parse command instead of manual parsing
def parse-text [text] {
  $text | split row ":" | each {|part| $part | str trim }
}

# prefer_direct_use: Import specific items instead of module
use std *

# redundant_ignore: Don't ignore useful output
def ignore-output [] {
  ls | ignore
  print "done" | ignore
}

# prefer_match_over_if_chain: Use match instead of if-else chain
def check-value [val] {
  if $val == 1 {
    "one"
  } else if $val == 2 {
    "two"
  } else if $val == 3 {
    "three"
  } else {
    "other"
  }
}

# max_positional_params: Too many positional parameters
def many-params [a b c d e f g] {
  [$a $b $c $d $e $f $g]
}

# remove_redundant_in: 'in' keyword is redundant
def check-membership [item list] {
  $item in $list
}

# Multiple violations in single function
def complex-violations [data] {
  mut result = []
  $data | each {|item|
    if $item > 0 {
      if $item < 100 {
        let processed = $item * 2
        $result = $result ++ [$processed]
      }
    }
  }
  let final = $result
  $final
}

# External commands that should use builtins
def external-heavy [] {
  ^ls /tmp | lines
  ^cat /etc/hosts | lines | where $it =~ "localhost"
  ^find . -type f | lines | first 20
  ^echo "test" | lines
  ^sort file.txt | lines
}

# prefer_where_over_for_if: Use where filter instead of for with if
def filter-loop [items] {
  mut filtered = []
  for item in $items {
    if $item.name == "test" {
      $filtered = $filtered ++ [$item]
    }
  }
  $filtered
}

# Nested loops for excessive nesting check
def deeply-nested [data] {
  for x in $data {
    for y in $x {
      for z in $y {
        if $z > 0 {
          if $z < 100 {
            print $z
          }
        }
      }
    }
  }
}

# unused_output: Command output is not used
def waste-output [] {
  ls
  1 + 1
  "hello" | str upcase
}

# Long function body for max_function_body_length check
def very-long-function [] {
  print "line 1"
  print "line 2"
  print "line 3"
  print "line 4"
  print "line 5"
  print "line 6"
  print "line 7"
  print "line 8"
  print "line 9"
  print "line 10"
  print "line 11"
  print "line 12"
  print "line 13"
  print "line 14"
  print "line 15"
  print "line 16"
  print "line 17"
  print "line 18"
  print "line 19"
  print "line 20"
  print "line 21"
  print "line 22"
  print "line 23"
  print "line 24"
  print "line 25"
  print "line 26"
  print "line 27"
  print "line 28"
  print "line 29"
  print "line 30"
  print "line 31"
  print "line 32"
  print "line 33"
  print "line 34"
  print "line 35"
  print "line 36"
  print "line 37"
  print "line 38"
  print "line 39"
  print "line 40"
  print "line 41"
  print "line 42"
  print "line 43"
  print "line 44"
  print "line 45"
  print "line 46"
  print "line 47"
  print "line 48"
  print "line 49"
  print "line 50"
  print "line 51"
  print "line 52"
  print "line 53"
  print "line 54"
  print "line 55"
}

# More external command violations
def more-external [] {
  ^head -20 somefile.txt | lines
  ^tail -30 logfile.log | lines
  ^wc -l file.txt | lines
  ^ps aux | lines
}

# Compound assignment opportunities
def more-compound [] {
  mut x = 10
  $x = $x + 5
  $x = $x - 3
  $x = $x * 2
  $x = $x / 2
  $x
}

# More collapsible ifs
def more-nested-ifs [a b c] {
  if $a > 0 {
    if $b > 0 {
      if $c > 0 {
        print "all positive"
      }
    }
  }
}

# Unnecessary variables
def chain-variables [] {
  let a = 1 + 2
  let b = $a + 3
  let c = $b + 4
  $c
}
