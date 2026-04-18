use std::io;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::os::fd::AsRawFd;

pub struct ClientSocket {
    stream: TcpStream,
}

pub enum ConnectionStatus {
    Established,
    Closed,
}

impl ClientSocket {
    pub fn accept_new_client(mut stream: TcpStream) -> io::Result<ClientSocket> {
        stream
            .set_nonblocking(true)
            .expect("Failed to set non-blocking mode on the stream");
        write_message(&mut stream, b"Hello from blocking server\n")?;
        Ok(ClientSocket { stream })
    }

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
                write_message(&mut self.stream, b"Blocking server received your message !\n")?;
                Ok(ConnectionStatus::Established)
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                Ok(ConnectionStatus::Established)
            }
            Err(e) => Err(e),
        }
    }
}

fn write_message(stream: &mut TcpStream, message: &[u8]) -> io::Result<()> {
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
