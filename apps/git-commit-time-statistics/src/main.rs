#[cfg(feature = "enabled")]
use git_commit_time_stat::cli_impl::{cli_main, MyResult};

#[cfg(feature = "enabled")]
fn main() -> MyResult<()> {
    cli_main()
}

#[cfg(not(feature = "enabled"))]
fn main() {
    println!("Crate is disabled.");
}
