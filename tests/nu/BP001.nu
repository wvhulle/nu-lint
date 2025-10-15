# BP001: Use error make for custom errors instead of print + exit
def bad-error [] {
    print "Error occurred"
    exit 1
}

def another-error [] {
    print "Something went wrong"
    exit 1
}
