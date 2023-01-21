use anyhow::anyhow;
use clap::ArgMatches;
use notify_rust::Timeout;
use sysinfo::{Pid, ProcessExt, ProcessRefreshKind, RefreshKind, System, SystemExt};

pub fn main(matches: &ArgMatches) -> anyhow::Result<()> {
    let pid = *matches.get_one::<u32>("pid").unwrap();

    let system = System::new_with_specifics(
        RefreshKind::default().with_processes(ProcessRefreshKind::default()),
    );

    let Some(process) = system.process(Pid::from(pid as usize)) else {
        return Err(anyhow!("Process with pid {} doesn't exist", pid));
    };

    process.wait();
    notify_rust::Notification::new()
        .summary(&format!("Process with pid {} exited", pid))
        .timeout(Timeout::Never)
        .show()?;
    Ok(())
}
