use clap::value_parser;

#[derive(clap::Parser, Debug)]
pub struct CliArgs {
    #[arg()]
    pub image1: String,
    #[arg()]
    pub image2: String,
    #[arg()]
    pub output: String,
    #[arg(long, value_parser = value_parser!(f64), default_value = "1.0")]
    pub image1_dim: f64,
    #[arg(long, value_parser = value_parser!(f64), default_value = "0.3")]
    pub image2_dim: f64,
}
