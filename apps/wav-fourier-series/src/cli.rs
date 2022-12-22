use clap::{value_parser, Arg, Command, ValueHint};

pub fn build_cli() -> Command {
    let cpu_num_string = num_cpus::get().to_string().into_boxed_str();
    let cpu_num_string: &str = Box::leak(cpu_num_string);

    Command::new("audio-fourier-series")
        .arg(
            Arg::new("src")
                .required(true)
                .value_hint(ValueHint::FilePath),
        )
        .arg(
            Arg::new("dest")
                .required(true)
                .value_hint(ValueHint::FilePath),
        )
        .arg(
            Arg::new("series-count")
                .required(true)
                .value_parser(value_parser!(u32)),
        )
        .arg(
            Arg::new("integral-segments-in-1s")
                .required(true)
                .value_parser(value_parser!(u32)),
        )
        .arg(
            Arg::new("thread-num")
                .required(false)
                .default_value(cpu_num_string)
                .value_parser(value_parser!(usize)),
        )
}
