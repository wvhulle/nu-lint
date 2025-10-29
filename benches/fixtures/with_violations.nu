# File with many external command violations

# ls violations
def list-files [] {
    ls | each { |file| $file.name }
    ls -la
    ls /tmp
}

# cat violations
def show-content [] {
    cat file.txt
    cat file1.txt file2.txt
    cat --number file.txt
}

# grep violations  
def search-text [] {
    grep "pattern" file.txt
    grep -i "pattern" file.txt
    grep -r "pattern" dir/
}

# find violations
def find-files [] {
    find . -name "*.rs"
    find /tmp -type f
    find . -name "test*"
}

# head violations
def show-first [] {
    head file.txt
    head -n 10 file.txt
    head -20 file.txt
}

# tail violations
def show-last [] {
    tail file.txt
    tail -n 10 file.txt
    tail -f log.txt
}

# sort violations
def sort-lines [] {
    sort file.txt
    sort -r file.txt
    sort -u file.txt
}

# uniq violations
def unique-lines [] {
    uniq file.txt
    uniq -c file.txt
    uniq -d file.txt
}

# Multiple violations in one function
def process-logs [] {
    cat /var/log/syslog | grep ERROR | sort | uniq
    ls /var/log | grep ".log" | head -10
    find /tmp -name "*.txt" | grep "test" | tail -5
}
