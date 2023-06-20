#[derive(clap::Parser, Debug)]
pub struct CliArgs {
    #[arg()]
    pub image1: String,
    #[arg()]
    pub image2: String,
    #[arg()]
    pub output: String,
}
