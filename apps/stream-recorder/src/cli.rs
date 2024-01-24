use std::path::PathBuf;

#[derive(Debug, clap::Parser)]
#[command(about = "A tool to recorde and replay stdin stream with the original speed")]
pub struct Args {
    #[arg(short, long, help = "Replay mode")]
    pub replay: bool,
    #[arg(short, long, help = "Also forward stdin to stdout")]
    pub forward: bool,
    #[arg(long, help = "Barely extract data from the file")]
    pub no_delay: bool,
    #[arg(short, long, help = "Skip # milliseconds when in reply mode")]
    pub skip: Option<u64>,
    #[arg(help = "File path to save or replay")]
    pub path: PathBuf,
}
