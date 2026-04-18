use std::io;
use std::net::TcpListener;

use std::os::fd::AsRawFd;
use rut_io_example::vfs::{ClientSocket, ConnectionStatus};

fn server_main(listener: TcpListener) -> io::Result<()> {
    println!("Blocking server started!");
    listener.set_nonblocking(true)?;

    let mut clients_socket: Vec<ClientSocket> = Vec::new();

    loop {
        loop {
            match listener.accept() {
                Ok((stream, addr)) => {
                    println!(
                        "Accepted connection from {addr:?} fd {}",
                        stream.as_raw_fd()
                    );
                    let socket = ClientSocket::accept_new_client(stream)?;
                    clients_socket.push(socket);
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) => {
                    println!("Accept error: {e:?}");
                    break;
                }
            }
        }

        let mut index = 0;
        while index < clients_socket.len() {
            let client = &mut clients_socket[index];
            match client.receive_data_and_respond() {
                Ok(ConnectionStatus::Established) => {
                    index += 1;
                }
                Ok(ConnectionStatus::Closed) => {
                    let fd = client.fd();
                    println!("Client {fd} disconnected");
                    clients_socket.swap_remove(index);
                }
                Err(e) => {
                    let fd = client.fd();
                    println!("Error reading from client: {} {:?}", fd, e);
                    clients_socket.swap_remove(index);
                }
            }
        }
    }
}

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8000")?;
    server_main(listener)?;
    Ok(())
}
