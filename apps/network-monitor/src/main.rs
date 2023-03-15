use anyhow::anyhow;
use network_monitor::cli::build_cli;
use network_monitor::interfaces_list;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use sysinfo::{NetworkExt, NetworksExt, RefreshKind, SystemExt};

fn main() -> anyhow::Result<()> {
    let matches = build_cli().get_matches();
    let list_interfaces = matches.get_flag("interfaces");

    if list_interfaces {
        for name in interfaces_list() {
            println!("{}", name);
        }
        return Ok(());
    }

    let interface = matches.get_one::<String>("interface").unwrap();

    let mut system = sysinfo::System::new_with_specifics(
        RefreshKind::new().with_networks().with_networks_list(),
    );
    loop {
        system.refresh_networks();
        let networks = system.networks();
        let Some(network_data) = networks.iter().find(|x| x.0 == interface).map(|x| x.1)
            else {
                return Err(anyhow!("Specified interface not found"));
        };

        println!(
            "{} {} {}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            network_data.total_transmitted(),
            network_data.total_received()
        );
        thread::sleep(Duration::from_secs(1));
    }
}