use std::path::PathBuf;

/// Represents stdin in wav format
#[derive(clap::Parser, Default, Clone)]
pub struct Args {
    /// Output sample rate
    #[arg(short = 'r', long)]
    pub sample_rate: u32,
    /// Output channel count
    #[arg(short, long, default_value = "2")]
    pub channels: u16,
    /// Input format
    ///
    /// Available formats are: s8, s16le, s16be, s32le, s32be, f32le, f32be
    #[arg(short, long, default_value = "s16le")]
    pub input_format: String,
    /// Output format
    ///
    /// Available formats are: s8, s16, s32, f32
    #[arg(short, long, default_value = "s16")]
    pub output_format: String,
    /// Output path
    pub output_file: PathBuf,
}
