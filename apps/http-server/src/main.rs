use clap::{Arg, Command};

use http_server::errors::*;

fn main() -> Result<()> {
    let matches = Command::new("http-server")
        .arg(
            Arg::new("port")
                .long("port")
                .short('p')
                .takes_value(true)
                .required(false)
                .help("The port to listen on")
                .default_value("8080"),
        )
        .arg(
            Arg::new("location")
                .required(false)
                .help("The location of the files to deploy")
                .default_value("."),
        )
        .get_matches();

    let port: u16 = matches.value_of("port").unwrap().parse()?;
    let location = matches.value_of("location").unwrap();
    http_server::server::run(port, location)
}
