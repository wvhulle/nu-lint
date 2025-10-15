# S002: Custom commands should use kebab-case naming convention
def myCommand [] {
    print "bad naming"
}

def my_command [] {
    print "underscore instead of hyphen"
}

def AnotherCommand [] {
    print "CamelCase"
}
