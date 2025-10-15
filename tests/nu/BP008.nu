# BP008: Prefer each pipeline over for loops
for item in [1 2 3 4 5] {
    echo ($item * 2)
}

for file in (ls | get name) {
    echo $file
}
