use std::fs::File;
use std::io::{stdin, stdout, BufReader, BufWriter, ErrorKind, Read, StdoutLock, Write};
use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

use crate::cli::Args;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use once_cell::sync::Lazy;

/// Stream log file format:
///
/// \[ Packet1 | Packet2 | ... \]
///
/// Packet format:
///
/// \[ Elapsed time in milliseconds (u32) | Data length (u16) | Data \]
///
/// The max `Data length` value is [`BUF_SIZE`]. All serialized integers are in little endianness.

const BUF_SIZE: usize = 8192;

pub mod cli;

pub fn record(path: &Path, forward: bool) -> anyhow::Result<()> {
    let mut stdin = stdin().lock();
    let start = SystemTime::now();
    let out = File::options()
        .truncate(true)
        .write(true)
        .read(true)
        .create(true)
        .open(path)?;
    let mut writer = BufWriter::new(out);
    let mut stdout: Lazy<StdoutLock> = Lazy::new(|| stdout().lock());

    let mut buf = [0_u8; BUF_SIZE];
    loop {
        let size = stdin.read(&mut buf)?;
        if size == 0 {
            break;
        }
        let elapsed = start.elapsed().unwrap().as_millis() as u32;
        writer.write_u32::<LE>(elapsed)?;
        writer.write_u16::<LE>(size as u16)?;
        writer.write_all(&buf[..size])?;
        writer.flush()?;
        if forward {
            stdout.write_all(&buf[..size])?;
            stdout.flush()?;
        }
    }

    Ok(())
}

pub fn replay(args: &Args) -> anyhow::Result<()> {
    let mut reader = BufReader::new(File::open(&args.path)?);

    let mut buf = [0_u8; BUF_SIZE];
    let start = SystemTime::now();
    let mut stdout = stdout().lock();
    loop {
        let elapsed = reader.read_u32::<LE>();
        let elapsed = match elapsed {
            Ok(e) => e,
            Err(e) => {
                if e.kind() == ErrorKind::UnexpectedEof {
                    break;
                } else {
                    return Err(e.into());
                }
            }
        };

        let size = reader.read_u16::<LE>()? as usize;
        reader.read_exact(&mut buf[..size])?;
        let delay = (start + Duration::from_millis(elapsed as u64))
            .duration_since(SystemTime::now())
            .unwrap_or(Duration::ZERO);
        if !args.no_delay {
            sleep(delay);
        }
        stdout.write_all(&buf[..size])?;
        stdout.flush()?;
    }
    Ok(())
}
