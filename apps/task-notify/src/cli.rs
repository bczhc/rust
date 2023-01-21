use clap::{value_parser, Arg, Command, ValueHint};

pub fn build_cli() -> Command {
    Command::new("task-notify")
        .subcommand(
            Command::new("watch").alias("w").arg(
                Arg::new("pid")
                    .value_parser(value_parser!(u32))
                    .required(true),
            ),
        )
        .subcommand(
            Command::new("run").alias("r").arg(
                Arg::new("command")
                    .required(true)
                    .num_args(1..)
                    .value_hint(ValueHint::CommandWithArguments)
                    .trailing_var_arg(true),
            ),
        )
        .subcommand_required(true)
}
