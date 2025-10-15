# BP009: Custom commands should have <= 2 positional parameters
def complex-command [
    param1: string
    param2: int
    param3: bool
    param4: string
] {
    print $param1
}

def too-many [a: int, b: int, c: int, d: int, e: int] {
    print $a
}
