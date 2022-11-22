use once_cell::sync::Lazy;

pub mod cli;

pub const TEST_INPUT_DATA: &str = include_str!("../../../lib/data/fourier-series-data.txt");
pub static CPU_NUM_STRING: Lazy<String> = Lazy::new(|| num_cpus::get().to_string());
