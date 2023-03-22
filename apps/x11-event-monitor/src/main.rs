#[cfg(target_os = "linux")]
fn main() {
    x11_event_monitor::linux::main()
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("Only support Linux platforms!");
}
