# BP004: Prefer parse command over manual string splitting
let line = "Device AA:BB:CC:DD:EE:FF MyDevice"
let parts = ($line | split row " ")
let mac = ($parts | get 1)
let name = ($parts | skip 2 | str join " ")

let data = "user:john:1000"
let fields = ($data | split row ":")
let username = ($fields | get 0)
