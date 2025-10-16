# BP014: Prefer builtin system commands

# Bad: env command
^env

# Bad: date command
^date

# Bad: man for help
^man ls

# Bad: read for input
^read -p "Enter value: "

# Bad: whoami
^whoami

# Bad: hostname
^hostname

# Bad: which command
^which ls

# Bad: pwd
^pwd

# Bad: cd
^cd /tmp

# Good: $env for environment
$env.PATH

# Good: date now
date now

# Good: help command
help ls

# Good: input
let name = input "Enter name: "

# Good: whoami builtin
whoami
