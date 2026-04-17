use std::io;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::os::fd::AsRawFd;

pub struct Connection {
    clients: Vec<TcpStream>,
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

    pub fn accept_new_client(&mut self, mut stream: TcpStream) -> io::Result<()> {
        stream
            .set_nonblocking(true)
            .expect("Failed to set non-blocking mode on the stream");
        send_message(&mut stream, b"Hello from blocking server\n")?;
        self.clients.push(stream);
        Ok(())
    }

    pub fn read_and_respond(&mut self) {
        let mut index = 0;
        while index < self.clients.len() {
            match receive_data_and_respond(&mut self.clients[index]) {
                Ok(ConnectionStatus::Established) => {
                    index += 1;
                }
                Ok(ConnectionStatus::Closed) => {
                    let fd = self.clients[index].as_raw_fd();
                    println!("Client {fd} disconnected");
                    self.clients.swap_remove(index);
                }
                Err(e) => {
                    let fd = self.clients[index].as_raw_fd();
                    println!("Error reading from client: {} {:?}", fd, e);
                    self.clients.swap_remove(index);
                }
            }
        }
    }
}

fn receive_data_and_respond(stream: &mut TcpStream) -> io::Result<ConnectionStatus> {
    let mut buf = [0u8; 1024];
    match stream.read(&mut buf) {
        Ok(0) => Ok(ConnectionStatus::Closed),
        Ok(n) => {
            println!(
                "Received {} from {}",
                String::from_utf8_lossy(&buf[..n]).replace('\n', ""),
                stream.as_raw_fd()
            );
            send_message(stream, b"Blocking server received your message !\n")?;
            Ok(ConnectionStatus::Established)
        }
        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => Ok(ConnectionStatus::Established),
        Err(e) => Err(e),
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
