use libc::read as unix_read;
use libc::write as unix_write;
use std::ffi::c_void;
use std::io;
use std::net::TcpStream;
use std::os::fd::{AsRawFd, IntoRawFd, RawFd};

pub enum ConnectionStatus {
    Established,
    Closed,
}

pub fn accept_new_client(stream: TcpStream) -> io::Result<RawFd> {
    stream
        .set_nonblocking(true)
        .expect("Failed to set non-blocking mode on the stream");
    let fd = stream.into_raw_fd();
    write(fd, b"Hello from blocking server\n")?;
    Ok(fd)
}

pub fn receive_data_and_respond(raw_fd: RawFd) -> io::Result<ConnectionStatus> {
    let mut buf = [0u8; 1024];
    let read = read(raw_fd, &mut buf);
    match read {
        Ok(0) => Ok(ConnectionStatus::Closed),
        Ok(n) => {
            println!(
                "Received {} from {}",
                String::from_utf8_lossy(&buf[..n]).replace('\n', ""),
                raw_fd
            );
            write(
                raw_fd.as_raw_fd(),
                b"Blocking server received your message !\n",
            )?;
            Ok(ConnectionStatus::Established)
        }
        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => Ok(ConnectionStatus::Established),
        Err(e) => {
            println!("Error reading from client 2: {} {:?}", raw_fd, e);
            Err(e)
        }
    }
}

fn write(fd: RawFd, message: &[u8]) -> io::Result<usize> {
    unsafe {
        let res = unix_write(fd, message.as_ptr() as *const c_void, message.len());
        if res >= 0 {
            Ok(res as usize)
        } else {
            Err(io::Error::last_os_error())
        }
    }
}

fn read(fd: RawFd, buf: &mut [u8]) -> io::Result<usize> {
    unsafe {
        let res = unix_read(fd, buf.as_mut_ptr() as *mut c_void, buf.len());
        if res >= 0 {
            Ok(res as usize)
        } else {
            Err(io::Error::last_os_error())
        }
    }
}
