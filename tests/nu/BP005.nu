# BP005: Check external command results consistently
def risky-external [] {
    let result = (^bluetoothctl info "AA:BB" | complete)
    print $result.stdout
}

def another-risky [] {
    let output = (^git status | complete)
    print $output.stdout
}
