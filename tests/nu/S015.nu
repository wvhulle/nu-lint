# S015: Unnecessary mut
# This file should be detected by the linter

# Bad: mut never reassigned
def process-data [input] {
    mut result = $input | str upcase
    echo $result
}

# Bad: multiple unnecessary muts
def calculate [] {
    mut x = 5
    mut y = 10
    echo ($x + $y)
}

# Good: mut is actually used for reassignment
def fibonacci [n: int] {
    mut a = 0
    mut b = 1
    for _ in 2..=$n {
        let c = $a + $b
        $a = $b
        $b = $c
    }
    $b
}

# Good: immutable variable
def get-value [] {
    let value = 42
    echo $value
}

# Bad: mut with no reassignment
def store-config [] {
    mut config = { key: "value" }
    echo $config
}

# Good: mut with compound assignment
def increment [] {
    mut counter = 0
    $counter += 1
    echo $counter
}
