use byteorder::{ReadBytesExt, WriteBytesExt};
use sha1::Sha1;
use std::io::{Cursor, ErrorKind, Read, Write};
use std::string::FromUtf8Error;

pub mod receive;
pub mod send;

pub mod lib {
    use bczhc_lib::io::{OpenOrCreate, ReadLine};
    use std::fs::{create_dir, create_dir_all, File};
    use std::io::{stdin, stdout, Read, Seek, SeekFrom, Write};
    use std::iter::{Map, Scan};
    use std::path::{Path, PathBuf};

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

    // configuration content:

    fn config_path() -> PathBuf {
        let mut config_dir = bczhc_config_dir().expect("Cannot found home directory to store");
        if !config_dir.exists() {
            create_dir_all(config_dir.clone()).unwrap();
        }
        config_dir.push("transfer");
        config_dir
    }

    fn open_config_file() -> File {
        File::open_append_file(config_path()).unwrap()
    }

    #[derive(Debug)]
    pub struct Configuration {
        pub destination_ip: String,
        pub port: u16,
    }

    pub fn handle_config() -> Configuration {
        let config_path = config_path();
        if !config_path.exists() {
            File::create(config_path.clone()).unwrap();
        }

        let mut destination_ip = None;
        let mut port = None;
        let mut matched = false;

        let mut conf_vec = read_config_file();
        for x in conf_vec.iter() {
            let key = &(*x).0;
            let value = &(*x).1;
            match key.as_str() {
                "destination-ip" => {
                    destination_ip = Some(value.clone());
                    matched = true;
                }
                "port" => {
                    port = Some(value.clone());
                    matched = true;
                }
                _ => {}
            }
        }

        if !matched {
            println!(
                "You need to enter some missing information, and then they will be stored at {}",
                config_path.to_str().unwrap()
            );
            let mut stdin = stdin();
            let mut stdout = stdout();

            if let None = destination_ip {
                print!("Destination IP: ");
                stdout.flush();
                destination_ip = Some(stdin.read_line_without_line_terminator().unwrap());
                conf_vec.push((
                    String::from("destination-ip"),
                    destination_ip.clone().unwrap().clone(),
                ));
            }

            if let None = port {
                print!("Port: ");
                stdout.flush();
                port = Some(stdin.read_line_without_line_terminator().unwrap());
                conf_vec.push((String::from("port"), port.clone().unwrap().clone()));
            }
            write_config_file(&conf_vec);
        }

        return Configuration {
            destination_ip: destination_ip.unwrap(),
            port: port.unwrap().parse().unwrap(),
        };
    }

    fn read_config_file() -> Vec<(String, String)> {
        let mut pairs = Vec::new();

        let mut config_file = open_config_file();
        loop {
            let result = config_file.read_line_without_line_terminator();
            if let Some(line) = result {
                let separator_index = line.find('=');
                if let None = separator_index {
                    continue;
                }
                let (key, value) = line.split_at(separator_index.unwrap());
                let value = &value[1..];

                let pair = (String::from(key), String::from(value));
                pairs.push(pair);
            } else {
                break;
            }
        }
        pairs
    }

    fn write_config_file(pairs: &Vec<(String, String)>) {
        let mut config_file = open_config_file();

        if config_file.metadata().unwrap().len() != 0 {
            config_file.seek(SeekFrom::End(-1)).unwrap();
            let mut last_byte = [0_u8];
            config_file.read_exact(&mut last_byte).unwrap();
            if last_byte[0] != b'\n' {
                config_file.write_all(b"\n").unwrap();
                config_file.seek(SeekFrom::Current(-1));
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
    }

    pub fn split_ipv4_string(ip: &String) -> Option<(u8, u8, u8, u8)> {
        let split = ip.split(".");
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
    for i in 0..5 {
        header[i] = HEADER_PREFIX[i];
    }
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
    if let None = result {
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
    InvalidIpv4
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
