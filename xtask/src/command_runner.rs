use std::{
    env, io,
    process::{Command, ExitStatus},
};

pub fn required_variable(variable_name: &str) -> io::Result<String> {
    env::var(variable_name)
        .map_err(|_| io::Error::other(format!("environment variable {variable_name} is not set")))
}

fn failed_command_error(program: &str, first_argument: &str, details: &str) -> io::Error {
    io::Error::other(format!("{program} {first_argument} failed: {details}"))
}

pub fn run_checked(program: &str, arguments: &[&str]) -> io::Result<String> {
    let output = Command::new(program).args(arguments).output()?;
    if !output.status.success() {
        return Err(failed_command_error(
            program,
            arguments.first().copied().unwrap_or_default(),
            &String::from_utf8_lossy(&output.stderr),
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn checked_status(command: &Command, status: ExitStatus) -> io::Result<()> {
    if status.success() {
        return Ok(());
    }
    let first_argument = command
        .get_args()
        .next()
        .map_or_else(String::new, |argument| {
            argument.to_string_lossy().into_owned()
        });
    Err(failed_command_error(
        &command.get_program().to_string_lossy(),
        &first_argument,
        &status.to_string(),
    ))
}

pub fn run_streaming(command: &mut Command) -> io::Result<()> {
    let status = command.status()?;
    checked_status(command, status)
}
