use std::io::{stdin, Error, Read, BufReader};
use termios::{tcflush, tcsetattr, Termios, ECHO, ICANON, TCIOFLUSH, TCSANOW, VMIN};
use bczhc_lib::io::TryReadExact;

struct MyStdin {
    stdin_fd: i32,
}

impl Read for MyStdin {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        unsafe {
            let read = libc::read(
                self.stdin_fd,
                buf.as_mut_ptr() as *mut libc::c_void,
                buf.len(),
            );
            if read == -1 {
                Err(std::io::Error::from(std::io::ErrorKind::Other))
            } else {
                Ok(read as usize)
            }
        }
    }
}

fn my_stdin() -> MyStdin {
    MyStdin { stdin_fd: 0 }
}

fn main() {
    // set_termios();

    let mut stdin = stdin();
    let mut buf = [0_u8; 5];
    loop {
        let result = stdin.try_read_exact(&mut buf);
        match result {
            Ok(r) => {
                if r == 0 {
                    break;
                }
                println!("{}", r);
                println!("{}", r);
            }
            Err(e) => {
                panic!(e.to_string());
            }
        }
    }
}

fn set_termios() {
    let mut termios = Termios::from_fd(1).unwrap();
    termios.c_lflag &= !(ICANON | ECHO);
    termios.c_cc[VMIN] = 1;
    tcsetattr(1, TCSANOW, &termios).unwrap();
    tcflush(1, TCIOFLUSH).unwrap();
}
