use crate::client_socket::ConnectionStatus::{Closed, Established};
use libc::pollfd;
use std::collections::HashMap;
use std::io;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::os::fd::{AsRawFd, RawFd};

pub struct Connection {
    clients: HashMap<RawFd, TcpStream>,
    pub poll_fds: Vec<pollfd>,
}

pub enum ConnectionStatus {
    Established,
    Closed,
}

impl Connection {
    pub fn new(accept_fd: pollfd) -> Self {
        Connection {
            clients: HashMap::new(),
            poll_fds: vec![accept_fd],
        }
    }

    pub fn accept_new_client(&mut self, fd: RawFd, mut stream: TcpStream, poll_fd: pollfd) {
        stream
            .set_nonblocking(true)
            .expect("Failed to set non-blocking mode on the stream");
        stream
            .write_all(b"Hello from poll server\n")
            .expect("Failed to send data to client");
        self.clients.insert(fd, stream);
        self.poll_fds.push(poll_fd);
    }

    pub fn receive_data_and_respond(&mut self, fd: &RawFd) -> Option<ConnectionStatus> {
        if let Some(stream) = self.clients.get_mut(fd) {
            let mut buf = [0; 1024];
            match stream.read(&mut buf) {
                Ok(0) => {
                    self.clients.remove(fd)?;
                    self.poll_fds.retain(|poll_fd| poll_fd.fd != fd.as_raw_fd());
                    Some(Closed)
                }
                Ok(n) => {
                    print!(
                        "Received from {}: {}",
                        fd,
                        String::from_utf8_lossy(&buf[..n])
                    );
                    stream
                        .write_all(b"Poll server recieved your message\n")
                        .unwrap();
                    Some(Established)
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => None,
                Err(_) => {
                    self.clients.remove(fd)?;
                    self.poll_fds.retain(|poll_fd| poll_fd.fd != fd.as_raw_fd());
                    Some(Closed)
                }
            }
        } else {
            None
        }
    }
}
