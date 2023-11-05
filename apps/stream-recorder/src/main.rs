use clap::Parser;
use stream_recorder::cli::Args;
use stream_recorder::{record, replay};

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if args.replay {
        replay(&args)?
    } else {
        record(&args.path, args.forward)?
    };
    Ok(())
}
