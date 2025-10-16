# S014: Completion functions should use 'nu-complete' prefix
def complete-branches [] {
  ^git branch | lines
}

def git-completion [] {
  ^git --help
}
