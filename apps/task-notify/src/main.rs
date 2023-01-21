use std::process::ExitCode;

use task_notify::cli::build_cli;

fn main() -> anyhow::Result<ExitCode> {
    let matches = build_cli().get_matches();
    let exit_code = if let Some(matches) = matches.subcommand_matches("run") {
        let status = task_notify::run::main(matches)?;
        ExitCode::from(status.code().unwrap() as u8)
    } else if let Some(matches) = matches.subcommand_matches("watch") {
        task_notify::watch::main(matches)?;
        ExitCode::SUCCESS
    } else {
        unreachable!();
    };
    Ok(exit_code)
}
