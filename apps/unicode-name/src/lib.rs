use std::io;
use std::io::BufRead;
use std::path::Path;
use std::sync::Mutex;

use once_cell::sync::Lazy;
use rusqlite::{params, Connection};
use utf8_chars::{BufReadCharsExt, Chars};

pub mod cli;

pub static CONFIG: Lazy<Mutex<Option<Config>>> = Lazy::new(|| Mutex::new(None));
static UCD_DB: Lazy<Mutex<UcdDatabase>> = Lazy::new(|| {
    let guard = CONFIG.lock().unwrap();
    let database_path = &guard.as_ref().unwrap().ucd_database;
    let connection = UcdDatabase::new(database_path).unwrap();
    Mutex::new(connection)
});

fn char_name(c: char) -> Option<String> {
    // `na` UCD properties aren't present for these characters
    let table = [
        "NULL",
        "START OF HEADING",
        "START OF TEXT",
        "END OF TEXT",
        "END OF TRANSMISSION",
        "ENQUIRY",
        "ACKNOWLEDGE",
        "BELL",
        "BACKSPACE",
        "CHARACTER TABULATION",
        "LINE FEED (LF)",
        "LINE TABULATION",
        "FORM FEED (FF)",
        "CARRIAGE RETURN (CR)",
        "SHIFT OUT",
        "SHIFT IN",
        "DATA LINK ESCAPE",
        "DEVICE CONTROL ONE",
        "DEVICE CONTROL TWO",
        "DEVICE CONTROL THREE",
        "DEVICE CONTROL FOUR",
        "NEGATIVE ACKNOWLEDGE",
        "SYNCHRONOUS IDLE",
        "END OF TRANSMISSION BLOCK",
        "CANCEL",
        "END OF MEDIUM",
        "SUBSTITUTE",
        "ESCAPE",
        "INFORMATION SEPARATOR FOUR",
        "INFORMATION SEPARATOR THREE",
        "INFORMATION SEPARATOR TWO",
        "INFORMATION SEPARATOR ONE",
    ];
    match c as u32 {
        0..=0x1F => Some(String::from(table[c as usize])),
        0x7F => Some(String::from("DELETE")),
        _ => {
            let guard = UCD_DB.lock().unwrap();
            let Some(json) = guard.query_json(c as u32).unwrap() else {
                return None
            };
            if json.is_empty() {
                return None;
            }
            let json: serde_json::Value = serde_json::from_str(&json).unwrap();
            for x in json.as_array().unwrap() {
                let group = x.as_array().unwrap();
                let key = group[0].as_str().unwrap();
                if key == "na" {
                    return Some(String::from(group[1].as_str().unwrap()));
                }
            }
            unreachable!()
        }
    }
}

#[inline]
fn char_block(c: char) -> Option<&'static str> {
    unic_ucd::block::Block::of(c).map(|x| x.name)
}

pub struct CharInfoIter<'a, R>
where
    R: BufRead,
{
    chars: Chars<'a, R>,
    char_offset: usize,
    byte_offset: usize,
}

impl<'a, R> CharInfoIter<'a, R>
where
    R: BufRead,
{
    pub fn new(reader: &'a mut R) -> Self {
        let chars = reader.chars();
        Self {
            chars,
            char_offset: 0,
            byte_offset: 0,
        }
    }
}

pub struct CharInfo {
    pub char: char,
    pub name: Option<String>,
    pub block: Option<&'static str>,
    pub char_offset: usize,
    pub byte_offset: usize,
}

impl<'a, R> Iterator for CharInfoIter<'a, R>
where
    R: BufRead,
{
    type Item = io::Result<CharInfo>;

    fn next(&mut self) -> Option<Self::Item> {
        let char = self.chars.next()?;
        match char {
            Ok(char) => {
                let name = char_name(char);
                let block = char_block(char);
                let r = Some(Ok(CharInfo {
                    char,
                    name,
                    block,
                    char_offset: self.char_offset,
                    byte_offset: self.byte_offset,
                }));
                self.byte_offset += char.len_utf8();
                self.char_offset += 1;
                r
            }
            Err(e) => {
                // IO error
                Some(Err(e))
            }
        }
    }
}

pub fn char_encoded_string(c: char) -> String {
    let utf8_len = c.len_utf8();
    let mut buf = [0_u8; 4];
    c.encode_utf8(&mut buf);

    let mut r = String::new();
    use std::fmt::Write;
    for x in buf[..utf8_len].iter() {
        write!(r, "{:02X} ", x).unwrap();
    }
    r.remove(r.len() - 1);
    r
}

pub fn char_utf32_string(c: char) -> String {
    format!("{:06X}", c as u32)
}

pub struct Config {
    pub ucd_database: String,
}

struct UcdDatabase {
    connection: Connection,
}

impl UcdDatabase {
    fn new<P: AsRef<Path>>(path: P) -> rusqlite::Result<Self> {
        let connection = Connection::open(path)?;
        Ok(Self { connection })
    }

    fn query_json(&self, codepoint: u32) -> rusqlite::Result<Option<String>> {
        let mut statement = self
            .connection
            .prepare("SELECT json FROM ucd WHERE codepoint IS ?")?;
        let mut rows = statement.query_map(params![codepoint], |row| {
            let json: String = row.get(0).unwrap();
            Ok(json)
        })?;
        match rows.next() {
            None => Ok(None),
            Some(Ok(json)) => Ok(Some(json)),
            _ => unreachable!(),
        }
    }
}
