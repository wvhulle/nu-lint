# S010: Use is-not-empty instead of not is-empty
if not ($list | is-empty) {
    print "has items"
}

let has_data = not ($data | is-empty)
