use std::io;
use std::net::TcpListener;

use rut_io_example::vfs::{
    accept_new_client, receive_data_and_respond, ConnectionStatus,
};
use std::os::fd::{AsRawFd, RawFd};

fn server_main(listener: TcpListener) -> io::Result<()> {
    println!("Blocking server started!");
    listener.set_nonblocking(true)?;

    let mut clients_socket: Vec<RawFd> = Vec::new();

    loop {
        loop {
            match listener.accept() {
                Ok((stream, addr)) => {
                    println!(
                        "Accepted connection from {addr:?} fd {}",
                        stream.as_raw_fd()
                    );
                    let socket = accept_new_client(stream)?;
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
            let client = clients_socket[index];
            match receive_data_and_respond(client) {
                Ok(ConnectionStatus::Established) => {
                    index += 1;
                }
                Ok(ConnectionStatus::Closed) => {
                    println!("Client {client} disconnected");
                    clients_socket.swap_remove(index);
                }
                Err(e) => {
                    println!("Error reading from client: {client} {e:?}");
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
