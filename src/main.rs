use std::{
    io::{self, ErrorKind, Write},
    process::ExitCode,
};

use nu_lint::cli::run;

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(error) if error.kind() == ErrorKind::BrokenPipe => ExitCode::SUCCESS,
        Err(error) => {
            let _ = writeln!(io::stderr(), "Error: {error}");
            ExitCode::FAILURE
        }
    }
}
