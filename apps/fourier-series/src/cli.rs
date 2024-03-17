use std::ffi::OsStr;

use clap::{value_parser, Arg, ArgAction, Command, ValueHint};

use crate::CPU_NUM_STRING;

pub fn build_cli() -> Command {
    Command::new("fourier-series")
        .arg(
            Arg::new("epicycle-count")
                .value_parser(value_parser!(u32))
                .default_value("100"),
        )
        .arg(
            Arg::new("period")
                .value_parser(value_parser!(f64))
                .default_value("100"),
        )
        .arg(
            Arg::new("thread-count")
                .value_parser(value_parser!(usize))
                .default_value(OsStr::new(&*CPU_NUM_STRING)),
        )
        .arg(
            Arg::new("integral-segments")
                .value_parser(value_parser!(u32))
                .default_value("1000000"),
        )
        .arg(
            Arg::new("integrator")
                .value_parser(value_parser!(Integrator))
                .short('i')
                .long("integrator")
                .default_value("trapezoid")
                .required(false)
                .help("Numerical integration method"),
        )
        .arg(
            Arg::new("data")
                .required(false)
                .short('d')
                .long("data")
                .help("Input data set file (if not provided, use a built-in test data set)")
                .value_hint(ValueHint::FilePath),
        )
        .arg(
            Arg::new("benchmark")
                .short('b')
                .long("benchmark")
                .help("Print the elapsed time after the calculation")
                .action(ArgAction::SetTrue),
        )
}

#[derive(Clone, clap::ValueEnum, Copy)]
pub enum Integrator {
    Trapezoid,
    LeftRectangle,
    RightRectangle,
    Simpson,
    Simpson38,
    Boole,
}
