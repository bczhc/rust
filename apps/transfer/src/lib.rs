use sha1::Sha1;
use std::fmt::Debug;
use std::io::Read;
use std::num::ParseIntError;
use std::string::FromUtf8Error;

pub mod config;
pub mod receive;
pub mod send;

pub mod lib {
    use crate::{check_option, Error, MyResult};
    use bczhc_lib::io::OpenOrCreate;
    use std::fs::File;
    use std::io::{Read, Seek, SeekFrom, Write};

    use std::path::PathBuf;

    #[inline]
    fn home_dir() -> Option<PathBuf> {
        home::home_dir()
    }

    #[inline]
    fn home_config_dir() -> Option<PathBuf> {
        let mut dir = home_dir()?;
        dir.push(".config");
        Some(dir)
    }

    #[inline]
    fn bczhc_config_dir() -> Option<PathBuf> {
        let mut dir = home_config_dir()?;
        dir.push("bczhc");
        Some(dir)
    }

    #[inline]
    fn config_file_path() -> Option<PathBuf> {
        let mut path = bczhc_config_dir()?;
        path.push("transfer");
        Some(path)
    }

    pub fn read_config_file() -> MyResult<Vec<(String, String)>> {
        check_config_file()?;

        let file_path = config_file_path();
        if file_path.is_none() {
            return Err(Error::CannotGetHomeDir);
        }

        let mut vec = Vec::new();

        let mut file = File::open(file_path.unwrap())?;
        let mut read_str = String::new();
        file.read_to_string(&mut read_str)?;

        for line in read_str.lines() {
            let find = line.find('=');
            match find {
                None => {
                    continue;
                }
                Some(find) => {
                    let split = line.split_at(find);
                    let key = split.0;
                    let value = &split.1[1..];
                    vec.push((String::from(key), String::from(value)));
                }
            }
        }

        Ok(vec)
    }

    pub fn write_config_file(pairs: &[(String, String)]) -> MyResult<()> {
        check_config_file()?;

        let result = config_file_path();
        let config_path = check_option(result, Error::CannotGetHomeDir)?;
        let mut config_file = File::create(config_path)?;

        if config_file.metadata().unwrap().len() != 0 {
            config_file.seek(SeekFrom::End(-1)).unwrap();
            let mut last_byte = [0_u8];
            config_file.read_exact(&mut last_byte).unwrap();
            if last_byte[0] != b'\n' {
                config_file.write_all(b"\n").unwrap();
                config_file.seek(SeekFrom::Current(-1)).unwrap();
            }
        };

        for (key, value) in pairs.iter() {
            let mut write_str = String::new();
            write_str.push_str(key);
            write_str.push('=');
            write_str.push_str(value);

            config_file.write_all(write_str.as_bytes()).unwrap();
            config_file.write_all(b"\n").unwrap();
        }

        Ok(())
    }

    fn check_config_file() -> MyResult<()> {
        let config_path = config_file_path();
        let config_path = check_option(config_path, Error::CannotGetHomeDir)?;
        if !config_path.exists() {
            File::open_or_create(config_path)?;
        }

        Ok(())
    }

    pub fn search_config<'a>(vec: &'a Vec<(String, String)>, key: &str) -> Option<&'a String> {
        for (k, v) in vec {
            if k.as_str() == key {
                return Some(v);
            }
        }
        None
    }

    pub fn search_config_index(vec: &[(String, String)], key: &str) -> Option<usize> {
        for (i, (k, _)) in vec.iter().enumerate() {
            if k.as_str() == key {
                return Some(i);
            }
        }
        None
    }

    pub fn split_ipv4_string(ip: &str) -> Option<(u8, u8, u8, u8)> {
        let split = ip.split('.');
        let split: Vec<&str> = split.collect();
        if split.len() != 4 {
            None
        } else {
            let a = split[0].parse();
            let b = split[1].parse();
            let c = split[2].parse();
            let d = split[3].parse();
            if a.is_err() || b.is_err() || c.is_err() || d.is_err() {
                return None;
            }

            Some((a.unwrap(), b.unwrap(), c.unwrap(), d.unwrap()))
        }
    }
}

pub fn compute_sha1_with_str(data: &[u8], extra: &str) -> [u8; 20] {
    let mut sha1 = Sha1::new();
    sha1.update(data);
    sha1.update(extra.as_bytes());
    sha1.digest().bytes()
}

pub fn compute_sha1(data: &[u8]) -> [u8; 20] {
    let mut sha1 = Sha1::new();
    sha1.update(data);
    sha1.digest().bytes()
}

#[derive(Debug)]
pub enum Type {
    File,
    Directory,
    Stdin,
    End,
}

impl Type {
    fn value(&self) -> u8 {
        match self {
            Type::File => 0,
            Type::Directory => 1,
            Type::Stdin => 2,
            Type::End => 3,
        }
    }

    fn value_of(value: u8) -> Option<Type> {
        match value {
            0 => Some(Type::File),
            1 => Some(Type::Directory),
            2 => Some(Type::Stdin),
            3 => Some(Type::End),
            _ => None,
        }
    }
}

pub fn make_header(file_type: Type) -> [u8; 8] {
    let mut header = [0_u8; 8];
    header[..5].copy_from_slice(&HEADER_PREFIX[..5]);
    header[5] = file_type.value();
    header
}

#[derive(Debug)]
pub struct Header {
    file_type: Type,
}

pub fn read_header<R>(input: &mut R) -> MyResult<Header>
where
    R: Read,
{
    let mut data = [0_u8; 8];
    input.read_exact(&mut data)?;
    if &data[..5] != HEADER_PREFIX {
        return Err(Error::InvalidHeader);
    }

    let result = Type::value_of(data[5]);
    if result.is_none() {
        return Err(Error::InvalidType);
    }
    let file_type = result.unwrap();

    Ok(Header { file_type })
}

const HEADER_PREFIX: &[u8; 5] = b"bczhc";

pub type MyResult<T> = Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    UnexpectedEOF,
    InvalidHeader,
    InvalidType,
    UnknownSubcommand,
    String(String),
    IOError(std::io::Error),
    Utf8Error(FromUtf8Error),
    InvalidUTF8,
    DigestCheckError,
    InvalidIpv4,
    Unsupported,
    CannotGetHomeDir,
    NoConfigKey(String),
    NoConfig(Configs),
    InvalidPort(ParseIntError),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IOError(e)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(e: FromUtf8Error) -> Self {
        Error::Utf8Error(e)
    }
}

#[derive(Debug)]
pub enum Configs {
    DestinationIP,
    Port,
}

impl Configs {
    pub fn key(self) -> &'static str {
        match self {
            Configs::DestinationIP => "destination-ip",
            Configs::Port => "port",
        }
    }

    pub fn value_of(key: &str) -> Option<Self> {
        match key {
            "destination-ip" => Some(Self::DestinationIP),
            "port" => Some(Self::Port),
            _ => None,
        }
    }
}

pub fn check_option<R>(option: Option<R>, error: Error) -> MyResult<R> {
    match option {
        None => Err(error),
        Some(some) => Ok(some),
    }
}

pub fn parse_port_str(port: &str) -> MyResult<u16> {
    let parse = port.parse::<u16>();
    match parse {
        Ok(r) => Ok(r),
        Err(e) => Err(Error::InvalidPort(e)),
    }
}
