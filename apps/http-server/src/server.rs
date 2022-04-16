use std::fs::File;
use std::io;
use std::io::{BufReader, Read, Write};
use std::net::{SocketAddrV4, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use std::thread::spawn;

use http::response::Builder;
use http::{HeaderMap, Response, StatusCode, Version};
use once_cell::sync::Lazy;
use regex::Regex;

use bczhc_lib::{rw_read, rw_write};

use crate::errors::*;
use crate::{CapitalizeHeader, HttpVersionAsStr};

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

            handle_request(request_header, stream)?;
            stream.flush()?;
            break;
        }
    }
    Ok(())
}

static HTTP_REQUEST_PATTERN_STR: &str = "^[a-zA-Z]* (.*) HTTP/.*$";
static HTTP_REQUEST_PATTERN: Lazy<RwLock<Regex>> =
    Lazy::new(|| RwLock::new(Regex::new(HTTP_REQUEST_PATTERN_STR).unwrap()));

fn handle_request(header: &str, stream: &mut TcpStream) -> Result<()> {
    let http_request_pattern = rw_read!(HTTP_REQUEST_PATTERN);
    let root_location_guard = rw_read!(ROOT_LOCATION);
    let root_location = root_location_guard.as_ref().unwrap().as_str();

    let lines = header.lines().collect::<Vec<_>>();

    if !http_request_pattern.is_match(lines[0]) {
        stream.write_response(Response::empty_body(StatusCode::BAD_REQUEST))?;
        return Ok(());
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
        stream.write_response(Response::empty_body(StatusCode::BAD_REQUEST))?;
        return Ok(());
    }

    let mut path_buf = PathBuf::from(root_location);
    let relative_path = format!(".{}", request_path);
    path_buf.push(&relative_path);
    let path = path_buf.as_path();

    if !path.exists() {
        stream.write_response(Response::empty_body(StatusCode::NOT_FOUND))?;
        return Ok(());
    }
    if path.is_dir() {
        let mut index1 = PathBuf::from(path);
        let mut index2 = PathBuf::from(path);
        index1.push("index.html");
        index2.push("index.htm");

        if index1.exists() && index1.is_file() {
            stream.write_response(Response::file_body(&index1))?;
            return Ok(());
        }
        if index2.exists() && index2.is_file() {
            stream.write_response(Response::file_body(&index2))?;
            return Ok(());
        }

        stream.write_response(Response::empty_body(StatusCode::FORBIDDEN))?;
        Ok(())
    } else if path.is_file() {
        stream.write_response(Response::file_body(path))?;
        Ok(())
    } else {
        stream.write_response(Response::empty_body(StatusCode::FORBIDDEN))?;
        Ok(())
    }
}

trait WriteHttp {
    fn write_head_line(&mut self, version: Version, status: StatusCode) -> io::Result<()>;

    fn write_headers(&mut self, headers: &HeaderMap) -> io::Result<()>;

    fn write_crlf(&mut self) -> io::Result<()>;
}

impl<W> WriteHttp for W
where
    W: Write,
{
    /// with ending CRLF
    fn write_head_line(&mut self, version: Version, status: StatusCode) -> io::Result<()> {
        let head_line = format!(
            "{} {} {}",
            version.as_str(),
            status.as_u16(),
            status.as_str()
        );
        self.write_all(head_line.as_bytes())?;
        self.write_crlf()
    }

    /// ending with two CRLFs (one empty line), meaning the body can be written immediately
    fn write_headers(&mut self, headers: &HeaderMap) -> io::Result<()> {
        for (name, value) in headers {
            let standard_name = name.to_capitalized();
            let header_line = format!("{}: {}", standard_name, value.to_str().unwrap());
            self.write_all(header_line.as_bytes())?;
            self.write_crlf()?;
        }
        self.write_crlf()
    }

    fn write_crlf(&mut self) -> io::Result<()> {
        self.write_all(b"\r\n")
    }
}

trait WriteHttpResponse<R> {
    fn write_response(&mut self, response: Response<R>) -> io::Result<()>;
}

impl<'a, W> WriteHttpResponse<FileBody<'a>> for W
where
    W: Write,
{
    fn write_response(&mut self, response: Response<FileBody<'a>>) -> io::Result<()> {
        self.write_head_line(response.version(), response.status())?;
        self.write_headers(response.headers())?;

        let file_path = response.body().path;
        let file = File::open(file_path)?;
        let mut reader = BufReader::new(file);
        io::copy(&mut reader, self)?;

        Ok(())
    }
}

impl<W> WriteHttpResponse<EmptyBody> for W
where
    W: Write,
{
    fn write_response(&mut self, response: Response<EmptyBody>) -> io::Result<()> {
        self.write_head_line(response.version(), response.status())?;
        self.write_headers(response.headers())
    }
}

#[derive(Copy, Clone)]
struct EmptyBody {}

static EMPTY_BODY: EmptyBody = EmptyBody {};

static HTTP_VERSION: Version = Version::HTTP_11;

trait DefaultBuilder {
    fn default_builder() -> Builder;
}

impl<T> DefaultBuilder for Response<T> {
    fn default_builder() -> Builder {
        Response::builder().version(HTTP_VERSION)
    }
}

trait BuildEmptyBody {
    fn empty_body(status: StatusCode) -> Response<EmptyBody>;
}

impl BuildEmptyBody for Response<EmptyBody> {
    fn empty_body(status: StatusCode) -> Response<EmptyBody> {
        Response::<EmptyBody>::default_builder()
            .status(status)
            .body(EMPTY_BODY)
            .unwrap()
    }
}

trait BuildFileBody {
    fn file_body(path: &Path) -> Response<FileBody>;
}

impl<'a> BuildFileBody for Response<FileBody<'a>> {
    fn file_body(path: &Path) -> Response<FileBody> {
        Response::<FileBody>::default_builder()
            .status(StatusCode::OK)
            .body(FileBody::new(path))
            .unwrap()
    }
}

struct FileBody<'a> {
    path: &'a Path,
}

impl<'a> FileBody<'a> {
    fn new(path: &'a Path) -> Self {
        FileBody { path }
    }
}
