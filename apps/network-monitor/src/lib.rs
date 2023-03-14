use sysinfo::{RefreshKind, SystemExt};

pub mod cli;

pub fn interfaces_list() -> Vec<String> {
    let system = sysinfo::System::new_with_specifics(
        RefreshKind::new().with_networks().with_networks_list(),
    );
    system
        .networks()
        .into_iter()
        .map(|x| x.0.clone())
        .collect::<Vec<_>>()
}
