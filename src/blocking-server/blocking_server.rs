mod client_socket;

use std::io;
use std::net::TcpListener;

use client_socket::{Connection, ConnectionStatus};
use std::os::fd::AsRawFd;

fn server_main(listener: TcpListener) -> io::Result<()> {
    println!("Blocking server started!");
    listener.set_nonblocking(true)?;
    let mut connections = Connection::new();

    loop {
        loop {
            match listener.accept() {
                Ok((stream, addr)) => {
                    println!(
                        "Accepted connection from {addr:?} fd {}",
                        stream.as_raw_fd()
                    );
                    connections.accept_new_client(stream)?;
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) => {
                    println!("Accept error: {e:?}");
                    break;
                }
            }
        }

        let mut clients_to_remove = Vec::new();

        for (index, client) in &mut connections {
            match client.receive_data_and_respond() {
                Ok(ConnectionStatus::Established) => {}
                Ok(ConnectionStatus::Closed) => {
                    let fd = client.fd();
                    println!("Client {fd} disconnected");
                    clients_to_remove.push(index);
                }
                Err(e) => {
                    let fd = client.fd();
                    println!("Error reading from client: {} {:?}", fd, e);
                    clients_to_remove.push(index);
                }
            }
        }

        connections.remove_clients_at(clients_to_remove);
    }
}

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8000")?;
    server_main(listener)?;
    Ok(())
}
