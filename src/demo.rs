fn main() {
window_size();
}

fn window_size() {
    let ws = libc::winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0
    };
    unsafe { libc::ioctl(1, libc::TIOCGWINSZ, &ws as *const libc::winsize) };

    println!("{}", ws.ws_col);
    println!("{}", ws.ws_row);
    println!("{}", ws.ws_xpixel);
    println!("{}", ws.ws_ypixel);
}
