use std::io;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::os::fd::AsRawFd;

pub struct ClientSocket {
    stream: TcpStream,
}

pub struct Connection {
    clients: Vec<ClientSocket>,
}

pub enum ConnectionStatus {
    Established,
    Closed,
}

impl Connection {
    pub fn new() -> Self {
        Connection {
            clients: Vec::new(),
        }
    }

    pub fn remove_clients_at(&mut self, mut indexes: Vec<usize>) {
        indexes.sort_unstable();

        for index in indexes.into_iter().rev() {
            self.clients.swap_remove(index);
        }
    }

    pub fn accept_new_client(&mut self, mut stream: TcpStream) -> io::Result<()> {
        stream
            .set_nonblocking(true)
            .expect("Failed to set non-blocking mode on the stream");
        send_message(&mut stream, b"Hello from blocking server\n")?;
        self.clients.push(ClientSocket { stream });
        Ok(())
    }
}

impl<'a> IntoIterator for &'a mut Connection {
    type Item = (usize, &'a mut ClientSocket);
    type IntoIter = std::iter::Enumerate<std::slice::IterMut<'a, ClientSocket>>;

    fn into_iter(self) -> Self::IntoIter {
        self.clients.iter_mut().enumerate()
    }
}

impl ClientSocket {
    pub fn fd(&self) -> i32 {
        self.stream.as_raw_fd()
    }

    pub fn receive_data_and_respond(&mut self) -> io::Result<ConnectionStatus> {
        let mut buf = [0u8; 1024];
        match self.stream.read(&mut buf) {
            Ok(0) => Ok(ConnectionStatus::Closed),
            Ok(n) => {
                println!(
                    "Received {} from {}",
                    String::from_utf8_lossy(&buf[..n]).replace('\n', ""),
                    self.fd()
                );
                send_message(&mut self.stream, b"Blocking server received your message !\n")?;
                Ok(ConnectionStatus::Established)
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                Ok(ConnectionStatus::Established)
            }
            Err(e) => Err(e),
        }
    }
}

fn send_message(stream: &mut TcpStream, message: &[u8]) -> io::Result<()> {
    match stream.write(message) {
        Ok(_) => Ok(()),
        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
            println!(
                "Skipping non-blocking write to {} because the socket is not ready",
                stream.as_raw_fd()
            );
            Ok(())
        }
        Err(e) => Err(e),
    }
}
