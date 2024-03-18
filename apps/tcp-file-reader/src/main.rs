use clap::Parser;
use std::fs::File;
use std::io;
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::thread::spawn;
use tcp_file_reader::cli::Args;

fn main() {
    let args = Args::parse();
    let listener = TcpListener::bind(format!("0.0.0.0:{}", args.port)).unwrap();
    loop {
        let (stream, from) = listener.accept().unwrap();
        println!("Connection from {}", from);
        let path = args.path.clone();
        spawn(move || {
            handle_stream(stream, path).unwrap();
        });
    }
}

fn handle_stream(mut stream: TcpStream, path: impl AsRef<Path>) -> anyhow::Result<()> {
    let mut file = File::open(path)?;
    io::copy(&mut file, &mut stream)?;
    drop(stream);

    Ok(())
}
