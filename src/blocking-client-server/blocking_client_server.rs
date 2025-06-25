use std::io;
use std::io::prelude::*;
use std::net::TcpListener;

fn server_main(listener: TcpListener) -> io::Result<()> {
    let mut n = 1;
    loop {
        let (mut socket, _) = listener.accept()?;
        // Using format! instead of write! avoids breaking up lines across multiple writes. This is
        // easier than doing line buffering on the client side.
        let mut buf = [0u8; 1024];
        match socket.read(&mut buf) {
            Ok(n) => {println!("read data {}", String::from_utf8_lossy(&buf[..n]))}
            Err(e) => {println!("Error occured {:?}", e)}
        };
        let greetings = format!("Hello You{n}\n");
        socket.write_all(greetings.as_bytes())?;
        n += 1;
    }
}


fn main() -> io::Result<()> {
    // Open the listener first, to avoid racing against the server thread.
    let listener = TcpListener::bind("0.0.0.0:8000")?;
    // Start the server on a background thread.
    server_main(listener).unwrap();
    Ok(())
}
