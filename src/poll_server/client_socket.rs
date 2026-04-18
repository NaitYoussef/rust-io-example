use crate::client_socket::ConnectionStatus::{Closed, Established};
use std::collections::HashMap;
use std::io;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::os::fd::RawFd;

pub struct Connection {
    clients: HashMap<RawFd, TcpStream>,
}

pub enum ConnectionStatus {
    Established,
    Closed,
}

impl Connection {
    pub fn new() -> Self {
        Connection {
            clients: HashMap::new(),
        }
    }

    pub fn accept_new_client(&mut self, fd: RawFd, mut stream: TcpStream) {
        stream
            .set_nonblocking(true)
            .expect("Failed to set non-blocking mode on the stream");
        stream
            .write_all(b"Hello from poll server\n")
            .expect("Failed to send data to client");
        self.clients.insert(fd, stream);
    }

    pub fn receive_data_and_respond(&mut self, fd: &RawFd) -> Option<ConnectionStatus> {
        if let Some(stream) = self.clients.get_mut(fd) {
            let mut buf = [0; 1024];
            match stream.read(&mut buf) {
                Ok(0) => {
                    self.clients.remove(fd)?;
                    Some(Closed)
                }
                Ok(n) => {
                    print!(
                        "Received from {}: {}",
                        fd,
                        String::from_utf8_lossy(&buf[..n])
                    );
                    stream
                        .write_all(b"Poll server received your message\n")
                        .unwrap();
                    Some(Established)
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => None,
                Err(_) => {
                    self.clients.remove(fd)?;
                    Some(Closed)
                }
            }
        } else {
            None
        }
    }
}
