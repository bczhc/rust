#[cfg(feature = "enabled")]
fn main() {
    x11_event_monitor::linux::main()
}

#[cfg(not(feature = "enabled"))]
fn main() {
    eprintln!("Crate is disabled.");
}
