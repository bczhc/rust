use cfg_if::cfg_if;
use clap::{Arg, Command};
use serial_cat::errors::*;
use serialport::DataBits;

fn main() -> Result<()> {
    let default_serial_port = if cfg!(windows) {
        "COM1"
    } else {
        "/dev/ttyUSB0"
    };

    let matches = Command::new("serial-cat")
        .arg(Arg::new("port").default_value(default_serial_port))
        .arg(Arg::new("baud").default_value("115200"))
        .get_matches();

    let port = matches.value_of("port").unwrap();
    let baud = matches
        .value_of("baud")
        .unwrap()
        .parse()
        .map_err(|_| Error::IllegalArgument)?;

    cfg_if! {
        if #[cfg(unix)] {
            let mut serial_port = serialport::new(port, baud)
            .data_bits(DataBits::Eight)
            .open_native()?;
            bczhc_lib::io::unix::attach_stream_to_stdio(&mut serial_port)?;
        } else {
            let mut serial_port = serialport::new(port, baud)
            .data_bits(DataBits::Eight)
            .open()?;
            generic::generic_pipe(&mut serial_port);
        }
    }

    Ok(())
}

#[cfg(not(unix))]
mod generic {
    use std::io::{stdin, stdout, Read, Write};
    use std::thread::{sleep, spawn};
    use std::time::Duration;

    pub(crate) fn generic_pipe<S>(stream: &mut S)
    where
        S: Write + Read,
    {
        let stream_addr = stream as *mut S as usize;
        let mut buf = [0_u8; 1024];
        let t1 = spawn(move || {
            let mut stdout = stdout().lock();
            let stream = unsafe { &mut *(stream_addr as *mut S) };

            loop {
                let result = stream.read(&mut buf);
                match result {
                    Ok(size) => {
                        stdout.write_all(&buf[0..size]).unwrap();
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {}
                    Err(e) => {
                        Err::<(), _>(e).unwrap();
                    }
                }
                sleep(Duration::from_millis(10));
            }
        });

        let t2 = spawn(move || {
            let mut stdin = stdin().lock();

            let stream = unsafe { &mut *(stream_addr as *mut S) };

            std::io::copy(&mut stdin, stream).unwrap();
        });

        t1.join().unwrap();
        t2.join().unwrap();
    }
}
