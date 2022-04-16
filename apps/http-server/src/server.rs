use std::fs::File;
use std::io::{Read, Write};
use std::net::{SocketAddrV4, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use std::thread::spawn;

use once_cell::sync::Lazy;
use regex::Regex;

use bczhc_lib::{rw_read, rw_write};

use crate::errors::*;

static ROOT_LOCATION: Lazy<RwLock<Option<String>>> = Lazy::new(|| RwLock::new(None));

pub fn run(port: u16, location: &str) -> Result<()> {
    rw_write!(ROOT_LOCATION).replace(String::from(location));

    let addr = SocketAddrV4::new("0.0.0.0".parse().unwrap(), port);
    let listener = TcpListener::bind(addr)?;

    loop {
        let (mut stream, client_addr) = listener.accept()?;
        println!("Accepted connection from {}", client_addr);
        spawn(move || {
            handle_connection(&mut stream).unwrap();
        });
    }
}

fn handle_connection(stream: &mut TcpStream) -> Result<()> {
    let mut buf = [0; 1];
    let mut read = Vec::new();
    loop {
        if stream.read(&mut buf)? == 0 {
            return Ok(());
        }
        read.push(buf[0]);
        if (read.len() > 4 && &read[(read.len() - 4)..] == b"\r\n\r\n")
            | (read.len() > 2 && &read[(read.len() - 2)..] == b"\n\n")
        {
            let request_header = std::str::from_utf8(&read).unwrap();
            println!("Request header: {request_header}");

            let response = handle_request(request_header);
            stream.write_all(&response)?;
            break;
        }
    }
    Ok(())
}

static HTTP_REQUEST_PATTERN_STR: &str = "^[a-zA-Z]* (.*) HTTP/.*$";
static HTTP_REQUEST_PATTERN: Lazy<RwLock<Regex>> =
    Lazy::new(|| RwLock::new(Regex::new(HTTP_REQUEST_PATTERN_STR).unwrap()));

fn handle_request(header: &str) -> Vec<u8> {
    let http_request_pattern = rw_read!(HTTP_REQUEST_PATTERN);
    let root_location_guard = rw_read!(ROOT_LOCATION);
    let root_location = root_location_guard.as_ref().unwrap().as_str();

    let lines = header.lines().collect::<Vec<_>>();

    if !http_request_pattern.is_match(lines[0]) {
        return construct_response_data(400, "Bad Request");
    }
    let raw_request_path = http_request_pattern
        .captures(lines[0])
        .unwrap()
        .get(1)
        .unwrap()
        .as_str();

    println!("Request: {}", raw_request_path);

    // get the request path before the query character '?'
    let request_path = raw_request_path
        .as_bytes()
        .iter()
        .position(|&b| b == b'?')
        .map(|p| &raw_request_path.as_bytes()[..p])
        .unwrap_or(raw_request_path.as_bytes());
    let request_path = unsafe { std::str::from_utf8_unchecked(request_path) };

    // URL decode
    let request_path = urlencoding::decode_binary(request_path.as_bytes());
    let request_path = unsafe { std::str::from_utf8_unchecked(&request_path) };

    if request_path.as_bytes()[0] != b'/' {
        return construct_response_data(400, "Bad Request");
    }

    let mut path_buf = PathBuf::from(root_location);
    let relative_path = format!(".{}", request_path);
    path_buf.push(&relative_path);
    let path = path_buf.as_path();

    if !path.exists() {
        return construct_response_data(404, "Not Found");
    }
    if path.is_dir() {
        let mut index1 = PathBuf::from(path);
        let mut index2 = PathBuf::from(path);
        index1.push("index.html");
        index2.push("index.htm");

        if index1.exists() && index1.is_file() {
            return construct_file_response_data(&index1);
        }
        if index2.exists() && index2.is_file() {
            return construct_file_response_data(&index2);
        }

        construct_response_data(403, "Forbidden")
    } else if path.is_file() {
        construct_file_response_data(path)
    } else {
        construct_response_data(403, "Forbidden")
    }
}

fn construct_file_response_data<P>(path: P) -> Vec<u8>
where
    P: AsRef<Path>,
{
    let file = File::open(path);
    if file.is_err() {
        return construct_response_data(500, "Internal Server Error");
    }

    let file = file.unwrap();
    let mut response_data = construct_response_data(200, "OK");

    let result = append_read_file_data(&mut response_data, file);
    if result.is_err() {
        return construct_response_data(500, "Internal Server Error");
    }
    response_data
}

fn construct_response_data(status: u16, msg: &str) -> Vec<u8> {
    format!("HTTP/1.1 {} {}\r\n\r\n", status, msg).into()
}

fn append_read_file_data(vec: &mut Vec<u8>, mut file: File) -> Result<()> {
    let mut buf = [0; 1024];
    loop {
        let read = file.read(&mut buf)?;
        if read == 0 {
            break;
        }
        vec.extend_from_slice(&buf[0..read]);
    }
    Ok(())
}
