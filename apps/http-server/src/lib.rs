use http::header::HeaderName;
use http::Version;

pub mod errors;
pub mod server;

pub trait HttpVersionAsStr {
    fn as_str(&self) -> &str;
}

impl HttpVersionAsStr for Version {
    fn as_str(&self) -> &str {
        match *self {
            Version::HTTP_09 => "HTTP/0.9",
            Version::HTTP_10 => "HTTP/1.0",
            Version::HTTP_11 => "HTTP/1.1",
            Version::HTTP_2 => "HTTP/2.0",
            Version::HTTP_3 => "HTTP/3.0",
            _ => unreachable!(),
        }
    }
}

pub trait CapitalizeHeader {
    fn to_capitalized(&self) -> String;
}

impl CapitalizeHeader for HeaderName {
    fn to_capitalized(&self) -> String {
        let name = self.as_str();
        let mut capitalized = Vec::with_capacity(name.len());
        let mut should_capitalize = false;

        for c in name.bytes() {
            if should_capitalize {
                capitalized.push(c.to_ascii_uppercase());
                should_capitalize = false;
            } else {
                capitalized.push(c);
            }
            if c == b'-' {
                should_capitalize = true;
            }
        }
        String::from_utf8(capitalized).unwrap()
    }
}
