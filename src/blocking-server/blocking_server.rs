use std::io;
use std::io::prelude::*;
use std::net::TcpListener;
use std::os::fd::AsRawFd;

const DISCONNECTED: usize = 0;

fn server_main(listener: TcpListener) -> io::Result<()> {
    println!("Blocking server started!");
    loop {
        let (mut stream, addr) = listener.accept()?;
        println!("Accepted connection from {addr:?}");
        stream.write_all("Blocking server received your message !\n".to_string().as_bytes())?;
        let mut buf = [0u8; 1024];
        loop {
            match stream.read(&mut buf) {
                Ok(DISCONNECTED) => {
                    println!("Client {} disconnected", stream.as_raw_fd());
                    break;
                }
                Ok(n) => {
                    println!("Received {} from {}", String::from_utf8_lossy(&buf[..n]), stream.as_raw_fd());
                    stream.write_all(&buf[..n])?;
                }
                Err(e) => {
                    println!("Error reading from client: {} {:?}",stream.as_raw_fd(), e);
                    break;
                }
            };
        }
    }
}

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8000")?;
    server_main(listener)?;
    Ok(())
}
