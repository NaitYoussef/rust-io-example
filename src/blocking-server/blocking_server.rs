mod client_socket;

use std::io;
use std::net::TcpListener;

use client_socket::Connection;
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

        connections.read_and_respond();
    }
}

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8000")?;
    server_main(listener)?;
    Ok(())
}
