use std::fmt::Debug;
use std::path::PathBuf;

#[derive(clap::Parser, Debug)]
/// Search binary in a file
pub struct Args {
    /// Patterns in hex string
    pub pattern: Vec<String>,
    /// Search in this file. If no file is given, stdin will be used
    #[arg(short, long)]
    pub file: Option<PathBuf>,
}
