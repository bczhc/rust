use clap::ArgMatches;
use notify_rust::Timeout;
use std::process::{Command, ExitStatus, Stdio};

pub fn main(matches: &ArgMatches) -> anyhow::Result<ExitStatus> {
    let command = matches.get_many::<String>("command").unwrap();
    let command = command.cloned().collect::<Vec<_>>();

    let mut process = Command::new(&command[0])
        .args(command.iter().skip(1))
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;
    let exit_status = process.wait()?;

    let body = if exit_status.success() {
        String::from("Command exited successfully")
    } else {
        format!(
            "Command exited with status code {}",
            exit_status.code().unwrap()
        )
    };

    notify_rust::Notification::new()
        .timeout(Timeout::Never)
        .summary(&command.join(" "))
        .body(&body)
        .show()?;

    Ok(exit_status)
}
