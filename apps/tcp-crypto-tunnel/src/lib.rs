use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::thread::spawn;

use polling::{Event, Poller};

use errors::*;

pub mod cli;
pub mod client;
pub mod errors;
pub mod server;

pub static mut KEY: [u8; 32] = [0_u8; 32];
pub static KEY_REF: &[u8; 32] = unsafe { &KEY };
pub const KEY_LEN: usize = 32;

pub fn prompt_password() -> Result<String> {
    let password = rpassword::prompt_password("enter password: ")?;
    if rpassword::prompt_password("enter again to verify: ")? != password {
        return Err(Error::PasswordNotMatch);
    }
    Ok(password)
}

pub fn derive_key<const S: usize>(password: &str) -> [u8; S] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(password.as_bytes());
    let mut output = hasher.finalize_xof();
    let mut buf = [0_u8; S];
    output.read_exact(&mut buf).unwrap();
    buf
}

pub fn crypto_interact_two_streams(stream1: &mut TcpStream, stream2: &mut TcpStream) -> Result<()> {
    let mut stream1_key_offset = 0_usize;
    let mut stream2_key_offset = 0_usize;
    let xor_key = *KEY_REF;

    let stream1_key = 0;
    let stream2_key = 1;

    let poller = Poller::new()?;
    poller.add(&*stream1, Event::readable(stream1_key))?;
    poller.add(&*stream2, Event::readable(stream2_key))?;

    let mut events = Vec::new();
    let mut buf = [0_u8; 4096];
    'poll_loop: loop {
        events.clear();
        poller.wait(&mut events, None)?;
        for ev in &events {
            let key = ev.key;
            let readable = ev.readable;
            match ev.key {
                _ if key == stream1_key && readable => {
                    let size = stream1.read(&mut buf)?;
                    if size == 0 {
                        break 'poll_loop;
                    }
                    for x in &mut buf[..size] {
                        *x ^= xor_key[stream1_key_offset % KEY_LEN];
                        stream1_key_offset += 1;
                    }
                    stream2.write_all(&buf[..size])?;
                }
                _ if key == stream2_key && readable => {
                    let size = stream2.read(&mut buf)?;
                    if size == 0 {
                        break 'poll_loop;
                    }
                    for x in &mut buf[..size] {
                        *x ^= xor_key[stream2_key_offset % KEY_LEN];
                        stream2_key_offset += 1;
                    }
                    stream1.write_all(&buf[..size])?;
                }
                _ => {
                    unreachable!();
                }
            }
            poller.modify(&*stream1, Event::readable(stream1_key))?;
            poller.modify(&*stream2, Event::readable(stream2_key))?;
        }
    }
    Ok(())
}

fn handle_connection(mut client: TcpStream, server_addr: &str) -> Result<()> {
    let mut server = TcpStream::connect(server_addr)?;
    crypto_interact_two_streams(&mut client, &mut server)?;
    Ok(())
}

pub fn generic_main(server_addr: String, listen_port: u16) -> Result<()> {
    let server_addr = Arc::new(server_addr);

    let listener = TcpListener::bind(SocketAddr::new("0.0.0.0".parse().unwrap(), listen_port))?;
    loop {
        let accepted = listener.accept()?;
        println!("Accepted: {}", accepted.1);
        let server_addr = Arc::clone(&server_addr);
        spawn(move || handle_connection(accepted.0, &server_addr).unwrap());
    }
}
