use libc::{poll, pollfd};
use std::io::{stdin, stdout};
use std::net::TcpStream;
use std::os::unix::io::{AsRawFd, RawFd};

pub mod connect;
pub mod errors;
pub mod listen;
use errors::*;

pub fn poll_between_two_streams(stream1: &mut TcpStream, stream2: &mut TcpStream) -> Result<()> {
    let stream1_fd = stream1.as_raw_fd();
    let stream2_fd = stream2.as_raw_fd();

    let routes = [
        PollRoute::new(stream1_fd, stream2_fd),
        PollRoute::new(stream2_fd, stream1_fd),
    ];
    loop_poll(&routes)
}

pub struct PollRoute {
    from: RawFd,
    to: RawFd,
}

impl PollRoute {
    fn new(from: RawFd, to: RawFd) -> PollRoute {
        Self { from, to }
    }
}

pub fn loop_poll(routes: &[PollRoute]) -> Result<()> {
    let mut fds = routes
        .iter()
        .map(|x| pollfd {
            fd: x.from as libc::c_int,
            events: libc::POLLIN,
            revents: 0,
        })
        .collect::<Vec<_>>();

    unsafe {
        'poll_loop: loop {
            let res = poll(fds.as_mut_ptr(), fds.len() as libc::nfds_t, 0);
            if res == -1 {
                return Err(errno_error());
            }
            let mut buf = [0_u8; 4096];
            for i in 0..fds.len() {
                if fds[i].revents == libc::POLLIN {
                    let from_fd = routes[i].from as libc::c_int;
                    let to_fd = routes[i].to as libc::c_int;

                    let size = libc::read(
                        from_fd,
                        buf.as_mut_ptr() as *mut libc::c_void,
                        buf.len() as libc::size_t,
                    );
                    if size == -1 {
                        return Err(errno_error());
                    }
                    if size == 0 {
                        break 'poll_loop;
                    }
                    let write_size = libc::write(
                        to_fd,
                        buf.as_mut_ptr() as *mut libc::c_void,
                        size as libc::size_t,
                    );
                    if write_size == -1 {
                        return Err(errno_error());
                    }
                    assert_eq!(write_size, size);
                }
            }
        }
    }

    Ok(())
}

pub fn attach_tcp_stream_to_stdio(stream: &TcpStream) -> Result<()> {
    let stdin = stdin().lock();
    let stdout = stdout().lock();

    let stdin_fd = stdin.as_raw_fd();
    let stdout_fd = stdout.as_raw_fd();

    let stream_fd = stream.as_raw_fd();

    let poll_routes = [
        PollRoute::new(stream_fd, stdout_fd),
        PollRoute::new(stdin_fd, stream_fd),
    ];

    loop_poll(&poll_routes)
}

fn errno_error() -> Error {
    Error::Poll(errno::errno())
}
