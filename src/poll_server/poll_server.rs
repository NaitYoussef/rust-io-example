mod client_socket;

use crate::client_socket::{Connection, ConnectionStatus};

use libc::pollfd;
use std::io;
use std::net::TcpListener;
use std::os::fd::{AsRawFd, RawFd};

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
    let poll_fd = create_pool_fd(listener.as_raw_fd());
    let mut poll_fds = vec![poll_fd];
    let mut connections = Connection::new();
    println!("Poll server started !");
    loop {
        poll(&mut poll_fds);
        for pfd in &poll_fds.clone() {
            if (pfd.revents & libc::POLLIN) != 0 && pfd.fd == listener.as_raw_fd() {
                match listener.accept() {
                    Ok((stream, addr)) => {
                        println!("Accepted connection from {:?} fd {}", addr, stream.as_raw_fd());
                        let client_fd = create_pool_fd(stream.as_raw_fd());
                        poll_fds.push(client_fd);
                        connections.accept_new_client(stream.as_raw_fd(), stream);
                    }
                    Err(e) => {
                        println!("Accept error: {e:?}");
                    }
                }
            } else if (pfd.revents & libc::POLLIN) != 0 {
                match connections.receive_data_and_respond(&pfd.fd.as_raw_fd()) {
                    None | Some(ConnectionStatus::Established) => {}
                    Some(ConnectionStatus::Closed) => {
                        let fd = pfd.fd.as_raw_fd();
                        println!("Client {fd} disconnected");
                        poll_fds.retain(|poll_fd| poll_fd.fd != fd.as_raw_fd());
                    }
                }
            }
        }
    }
}

fn create_pool_fd(fd: RawFd) -> pollfd {
    pollfd {
        fd: fd.as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    }
}
