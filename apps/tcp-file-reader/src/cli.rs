use std::path::PathBuf;

#[derive(clap::Parser, Debug)]
pub struct Args {
    #[arg(short, long)]
    pub port: u16,
    pub path: PathBuf,
}
