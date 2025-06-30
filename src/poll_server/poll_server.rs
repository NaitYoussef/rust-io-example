use libc::{close, pollfd};
use std::collections::HashMap;
use std::io;
use std::io::prelude::*;
use std::net::TcpListener;
use std::os::fd::AsRawFd;

fn poll(fds: &mut Vec<pollfd>) {
    unsafe {
        libc::poll(
            fds.as_mut_ptr(),
            fds.len() as libc::nfds_t,
            -1 as libc::c_int,
        );
    }
}

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8000")?;
    listener.set_nonblocking(true)?;
    let pollfd = pollfd {
        fd: listener.as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    };
    let mut poll_fds = vec![pollfd];
    let mut client_streams = HashMap::new();
    let mut new_clients = Vec::new();
    let mut finished_clients = Vec::new();
    println!("Serveur poll démarré !");
    loop {
        poll(&mut poll_fds);
        for pfd in &poll_fds {
            if (pfd.revents & libc::POLLIN) != 0 && pfd.fd == listener.as_raw_fd() {
                match listener.accept() {
                    Ok((mut stream, addr)) => {
                        stream.write_all("Hello from poll server \n".to_string().as_bytes())?;
                        println!("Accepted connection from {:?}", addr);
                        let clientfd = pollfd {
                            fd: stream.as_raw_fd(),
                            events: libc::POLLIN,
                            revents: 0,
                        };
                        client_streams.insert(stream.as_raw_fd(), stream);
                        new_clients.push(clientfd);
                    }
                    Err(e) => {
                        println!("Accept error: {:?}", e);
                    }
                }
            } else if (pfd.revents & libc::POLLIN) != 0 {
                let mut buf = [0u8; 1024];
                let mut client_stream = client_streams.get(&pfd.fd.as_raw_fd()).unwrap();
                unsafe {
                    match client_stream.read(&mut buf) {
                        Ok(0) => {
                            // Le client a fermé la connexion
                            println!("Client {} disconnected", pfd.fd.as_raw_fd());
                            client_streams.remove(&pfd.fd.as_raw_fd());
                            finished_clients.push(pfd.fd.as_raw_fd());
                        }
                        Ok(n) => {
                            // On a reçu des données => les afficher + renvoyer un écho
                            println!(
                                "Received from {}: {}",
                                pfd.fd.as_raw_fd(),
                                String::from_utf8_lossy(&buf[..n])
                            );
                            client_stream.write_all("Hello you\n".as_bytes()).unwrap();
                        }
                        Err(e) => {
                            // Erreur de lecture => ferme le client
                            println!("Read error on {}: {:?}", pfd.fd.as_raw_fd(), e);
                            client_streams.remove(&pfd.fd.as_raw_fd());
                            // Test this ligne just added
                            close(pfd.fd.as_raw_fd());
                            finished_clients.push(pfd.fd.as_raw_fd())
                        }
                    }
                }
            }
        }
        poll_fds.retain(|fd| !finished_clients.contains(&fd.fd.as_raw_fd()));
        poll_fds.extend(new_clients.clone());
        new_clients.clear();
        finished_clients.clear();
    }
}
