use archive::cli::build_cli;

fn main() -> anyhow::Result<()> {
    let matches = build_cli().get_matches();

    if let Some(matches) = matches.subcommand_matches("create") {
        archive::create::main(matches)
    } else if let Some(matches) = matches.subcommand_matches("list") {
        archive::list::main(matches)
    } else if let Some(matches) = matches.subcommand_matches("extract") {
        archive::extract::main(matches)
    } else if let Some(matches) = matches.subcommand_matches("test") {
        archive::test::main(matches)
    } else if let Some(matches) = matches.subcommand_matches("info") {
        archive::info::main(matches)
    } else {
        unreachable!()
    }?;

    Ok(())
}
