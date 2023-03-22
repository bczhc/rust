#[cfg(unix)]
fn main() {
    x11_event_monitor::unix::main()
}

#[cfg(not(unix))]
fn main() {
    eprintln!("Only support *nix platforms!");
}
