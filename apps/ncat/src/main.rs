use std::io::{Cursor, Read};

use clap::{Arg, Command};

use ncat::errors::*;

fn main() -> Result<()> {
    let mut app = Command::new("ncat")
        .arg(
            Arg::new("listen-port")
                .short('l')
                .long("listen")
                .takes_value(true)
                .required(false)
                .conflicts_with("ip")
                .conflicts_with("port"),
        )
        .arg(Arg::new("ip").requires("port").required(false))
        .arg(Arg::new("port").requires("ip").required(false));

    let mut help = Cursor::new(Vec::new());
    app.write_help(&mut help).unwrap();
    let matches = app.get_matches();

    if !matches.args_present() {
        let mut help_str = String::new();
        help.set_position(0);
        help.read_to_string(&mut help_str).unwrap();
        print!("{}", help_str);
        return Ok(());
    }

    use ncat::{connect, listen};
    if matches.is_present("listen-port") {
        let listen_port: u16 = matches.value_of("listen-port").unwrap().parse()?;
        listen::listen(listen_port)
    } else if matches.is_present("ip") && matches.is_present("port") {
        let ip = matches.value_of("ip").unwrap();
        let port: u16 = matches.value_of("port").unwrap().parse()?;
        connect::connect(&format!("{}:{}", ip, port))
    } else {
        unreachable!()
    }
}
