use std::io;
use std::os::fd::AsRawFd;
use std::os::fd::BorrowedFd;

#[cfg(feature = "async")]
use std::os::fd::AsFd;

pub use libc::termios as Termios;

/// Get the current terminal attributes.
pub fn get_terminal_attr(fd: BorrowedFd) -> io::Result<Termios> {
    unsafe {
        let mut termios = std::mem::zeroed();
        if libc::tcgetattr(fd.as_raw_fd(), &mut termios) < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(termios)
    }
}

/// Set the terminal attributes.
pub fn set_terminal_attr(fd: BorrowedFd, termios: &Termios) -> io::Result<()> {
    if unsafe { libc::tcsetattr(fd.as_raw_fd(), libc::TCSANOW, termios) } < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// Modifies the termios to enable Raw Mode.
/// This uses `cfmakeraw` which is standard on Unix.
pub fn make_raw(termios: &mut Termios) {
    unsafe { libc::cfmakeraw(termios) };
}

/// Get terminal window size (cols, rows).
pub fn get_window_size(fd: BorrowedFd) -> io::Result<(u16, u16)> {
    unsafe {
        let mut winsize: libc::winsize = std::mem::zeroed();
        if libc::ioctl(fd.as_raw_fd(), libc::TIOCGWINSZ, &mut winsize) < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok((winsize.ws_col, winsize.ws_row))
    }
}

/// Check if input is available within a timeout (milliseconds).
pub fn poll_input(fd: BorrowedFd, timeout_ms: i32) -> io::Result<bool> {
    unsafe {
        let mut fds = libc::pollfd {
            fd: fd.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        let ret = libc::poll(&mut fds, 1, timeout_ms);
        if ret < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(ret > 0 && (fds.revents & libc::POLLIN) != 0)
    }
}

#[cfg(feature = "async")]
pub async fn poll_input_async<F: AsFd>(fd: F, timeout_ms: u32) -> io::Result<bool> {
    use std::time::Duration;
    use tokio::time::timeout;

    let fd = fd.as_fd();

    let result = timeout(Duration::from_millis(timeout_ms as u64), async {
        let mut buf = [0u8; 1];
        let mut stdin = std::io::stdin();
        loop {
            match stdin.read(&mut buf).await {
                Ok(0) => return false,
                Ok(_) => return true,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
                Err(_) => return false,
            }
        }
    }).await;

    match result {
        Ok(v) => Ok(v),
        Err(_) => Ok(false),
    }
}
