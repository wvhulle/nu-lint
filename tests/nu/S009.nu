# S009: Variable assigned and immediately returned
def get-value [] {
  let result = (some | pipeline)
  $result
}

def calculate [] {
  let answer = (42 | into string)
  $answer
}
