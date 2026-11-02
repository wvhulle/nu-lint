def count-to-ten [] {
    mut counter = 0
    loop {
        if $counter >= 10 {
            break
        }
        print $counter
        $counter = $counter + 1
    }
}

def main [] {
    count-to-ten
}
