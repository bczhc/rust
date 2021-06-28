#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use linux::main;
#[cfg(not(target_os = "linux"))]
mod others;
#[cfg(not(target_os = "linux"))]
use others::main;

fn main() -> Result<(), String> {
    main::run()
}
