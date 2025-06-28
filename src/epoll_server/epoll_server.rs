use libc::{epoll_create1, epoll_ctl, epoll_event, epoll_wait, EPOLLIN, EPOLLET, EPOLL_CTL_ADD, EPOLL_CTL_DEL, close, c_int};
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::os::fd::{AsRawFd, RawFd};

#[cfg(target_os = "linux")]
fn main() -> io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8000")?;
    listener.set_nonblocking(true)?;
    let epoll_fd = unsafe { epoll_create1(0) };
    if epoll_fd < 0 {
        return Err(io::Error::last_os_error());
    }

    let mut event = epoll_event {
        events: (EPOLLIN | EPOLLET) as u32,
        u64: listener.as_raw_fd() as u64,
    };
    unsafe {
        epoll_ctl(epoll_fd, EPOLL_CTL_ADD, listener.as_raw_fd(), &mut event);
    }

    let mut client_streams: HashMap<RawFd, TcpStream> = HashMap::new();
    let mut events = [epoll_event { events: 0, u64: 0 }; 1024];

    println!("Serveur epoll démarré !");
    loop {
        let nfds = unsafe { epoll_wait(epoll_fd, events.as_mut_ptr(), events.len() as i32, 10_000) };
        if nfds < 0 {
            return Err(io::Error::last_os_error());
        }
        for n in 0..nfds as usize {
            let fd = events[n].u64 as RawFd;
            if fd == listener.as_raw_fd() {
                // Nouvelle connexion
                loop {
                    match listener.accept() {
                        Ok((mut stream, addr)) => {
                            stream.set_nonblocking(true)?;
                            println!("Accepted connection from {addr:?}");
                            stream.write_all(b"Hello from epoll server\n")?;
                            let client_fd = stream.as_raw_fd();
                            let mut ev = epoll_event {
                                events: (EPOLLIN | EPOLLET) as u32,
                                u64: client_fd as u64,
                            };
                            unsafe {
                                epoll_ctl(epoll_fd, EPOLL_CTL_ADD, client_fd, &mut ev);
                            }
                            client_streams.insert(client_fd, stream);
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                            break;
                        }
                        Err(e) => {
                            println!("Accept error: {e:?}");
                            break;
                        }
                    }
                }
            } else {
                // Données à lire sur un client
                let mut buf = [0u8; 1024];
                if let Some(stream) = client_streams.get_mut(&fd) {
                    match stream.read(&mut buf) {
                        Ok(0) => {
                            println!("Client {fd} disconnected");
                            disconnect(epoll_fd, &mut client_streams, fd);
                        }
                        Ok(n) => {
                            println!("Received from {}: {}", fd, String::from_utf8_lossy(&buf[..n]));
                            stream.write_all(b"Hello you\n")?;
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                        Err(e) => {
                            println!("Read error on {fd}: {e:?}");
                            disconnect(epoll_fd, &mut client_streams, fd);
                        }
                    }
                }
            }
        }
    }
}

fn disconnect(epoll_fd: c_int, client_streams: &mut HashMap<RawFd, TcpStream>, fd: RawFd) {
    client_streams.remove(&fd);
    unsafe {
        epoll_ctl(epoll_fd, EPOLL_CTL_DEL, fd, std::ptr::null_mut());
        close(fd);
    }
}