use std::io;
use std::net::TcpListener;

use rut_io_example::vfs::{accept_new_client, close, receive_data_and_respond, ConnectionStatus};
use std::os::fd::{AsRawFd, RawFd};

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8000")?;
    println!("Blocking server started!");
    listener.set_nonblocking(true)?;

    let mut fds: Vec<RawFd> = Vec::new();

    loop {
        loop {
            match listener.accept() {
                Ok((stream, addr)) => {
                    println!(
                        "Accepted connection from {addr:?} fd {}",
                        stream.as_raw_fd()
                    );
                    let socket = accept_new_client(stream, "Hello from blocking server\n")?;
                    fds.push(socket);
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) => {
                    println!("Accept error: {e:?}");
                    break;
                }
            }
        }

        let mut index = 0;
        while index < fds.len() {
            let client = fds[index];
            match receive_data_and_respond(client, "Blocking server received your message !\n") {
                Ok(ConnectionStatus::Established) => {
                    index += 1;
                }
                Ok(ConnectionStatus::Closed) => {
                    println!("Client {client} disconnected");
                    let fd = fds.swap_remove(index);
                    close(fd);
                }
                Err(e) => {
                    println!("Error reading from client: {client} {e:?}");
                    let fd = fds.swap_remove(index);
                    close(fd);
                }
            }
        }
    }
}
