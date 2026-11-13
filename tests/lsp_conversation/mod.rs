pub fn framed(json_body: &str) -> String {
    format!("Content-Length: {}\r\n\r\n{json_body}", json_body.len())
}

pub fn shutdown_conversation() -> String {
    [
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"processId":null,"rootUri":null,"capabilities":{}}}"#,
        r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#,
        r#"{"jsonrpc":"2.0","id":2,"method":"shutdown","params":null}"#,
        r#"{"jsonrpc":"2.0","method":"exit","params":null}"#,
    ]
    .map(framed)
    .concat()
}
