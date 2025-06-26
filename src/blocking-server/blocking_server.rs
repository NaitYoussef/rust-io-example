use std::io;
use std::io::prelude::*;
use std::net::TcpListener;
use std::os::fd::AsRawFd;

fn server_main(listener: TcpListener) -> io::Result<()> {
    let mut n = 1;
    println!("Serveur bloquant démarré !");
    loop {
        let (mut stream, addr) = listener.accept()?;
        println!("Nouveau client : {} {}", addr, stream.as_raw_fd());
        // Using format! instead of write! avoids breaking up lines across multiple writes. This is
        // easier than doing line buffering on the client side.
        let mut buf = [0u8; 1024];
        stream.write_all("Hello from blocking server \n".to_string().as_bytes())?;
        loop {
            match stream.read(&mut buf) {
                Ok(0) => {
                    println!("Client déconnecté {}", stream.as_raw_fd());
                    break;
                }
                Ok(n) => {
                    println!("Reçu : {} from {}", String::from_utf8_lossy(&buf[..n]), stream.as_raw_fd());
                    stream.write_all(&buf[..n])?;
                }
                Err(e) => {
                    println!("Error occured {:?}", e);
                    break;
                }
            };
        }
        n += 1;
    }
}

fn main() -> io::Result<()> {
    // Open the listener first, to avoid racing against the server thread.
    let listener = TcpListener::bind("0.0.0.0:8000")?;
    // Start the server on a background thread.
    server_main(listener)?;
    Ok(())
}
