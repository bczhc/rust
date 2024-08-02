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
                .help("Webapp root location")
                .default_value("."),
        )
        .arg(
            Arg::new("ip")
                .long("ip")
                .takes_value(true)
                .required(false)
                .help("IP to bind")
                .default_value("0.0.0.0"),
        )
        .get_matches();

    let port: u16 = matches.value_of("port").unwrap().parse()?;
    let location = matches.value_of("location").unwrap();
    let ip = matches.value_of("ip").unwrap();
    http_server::server::run(port, location, ip)
}
