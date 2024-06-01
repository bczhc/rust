use std::fs::File;
use std::io;
use std::io::{BufReader, Read, stdin};
use std::process::exit;

use anyhow::anyhow;
use clap::Parser;

use bin_find::Args;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut patterns = args.pattern.iter().map(|x| hex::decode(x).map_err(|_|
        anyhow!("Invalid hex string: {}", x))).collect::<Result<Vec<_>, _>>()?;
    patterns.sort();
    patterns.dedup();

    let found = match args.file {
        None => search(stdin().lock(), &patterns)?,
        Some(path) => {
            let reader = BufReader::new(File::open(path)?);
            search(reader, &patterns)?
        }
    };

    if !found {
        exit(1);
    }
    Ok(())
}

fn search<R: Read>(mut reader: R, patterns: &[Vec<u8>]) -> io::Result<bool> {
    assert_ne!(patterns.len(), 0);
    let mut patterns = patterns.to_vec();
    // sort by pattern length
    patterns.sort_by_key(|x| x.len());

    // allocate the longest possible buffer
    let longest_size = patterns.last().unwrap().len();
    let mut buf = vec![0_u8; longest_size];
    // The initial read. If an extra one byte is read and append to this, a full window will be
    // made.
    reader.read_exact(&mut buf[..(longest_size - 1)]).unwrap();

    let mut found = false;
    let mut found_print = |pattern: &[u8], offset: usize| {
        found = true;
        println!("{}: {}", hex::encode(pattern), offset);
    };

    let mut offset = 0_usize;
    loop {
        // read one byte and fill the last item
        let result = reader.read_exact(&mut buf[(longest_size - 1)..longest_size]);
        match result {
            Ok(_) => {}
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                // reaches the EOF; do extra searches for those patterns
                // whose size is less than the longest one.
                let saved_offset = offset;
                let remain = &buf[..(longest_size - 1)];
                for i in (0..remain.len()).rev() {
                    for p in &patterns {
                        if remain[i..].starts_with(p) {
                            found_print(p, saved_offset + i);
                        }
                    }
                }
                break;
            }
            Err(e) => Err(e)?
        }
        // matching
        for p in &patterns {
            if buf.starts_with(p) {
                found_print(p, offset);
            }
        }
        buf.rotate_left(1);
        offset += 1;
    }
    Ok(found)
}

#[cfg(test)]
mod test {
    use std::io::Cursor;
    use crate::search;

    #[test]
    fn test() {
        // println!("{:?}\n", test_string("abcde", &["a"]));
        // println!("{:?}\n", test_string("abcde", &["ab"]));
        // println!("{:?}\n", test_string("abcde", &["bcd"]));
        // println!("{:?}\n", test_string("abcde", &["bcde"]));
        // println!("{:?}\n", test_string("abcde", &["bcdef"]));
        // println!("{:?}\n", test_string("abcde", &["de"]));
        // println!("{:?}\n", test_string("abcde", &["e"]));
        // println!("{:?}\n", test_string("abcde", &["ef"]));
        println!("{:?}\n", test_string("abcdefghijklm", &["def", "m", "lm"]));
    }

    fn test_string(haystack: &str, needle: &[&str]) -> std::io::Result<bool> {
        let reader = Cursor::new(haystack.as_bytes());
        let needle = needle.iter().map(|x| x.as_bytes().to_vec()).collect::<Vec<_>>();
        search(reader, &needle)
    }
}
