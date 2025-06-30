mod client_socket;

use crate::client_socket::{Connection, ConnectionStatus};

use libc::pollfd;
use std::io;
use std::io::prelude::*;
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
    let mut connections = Connection::new(poll_fd);
    println!("Poll server started !");
    loop {
        let mut vec = connections.poll_fds.clone();
        poll(&mut vec);
        for pfd in &vec {
            if (pfd.revents & libc::POLLIN) != 0 && pfd.fd == listener.as_raw_fd() {
                match listener.accept() {
                    Ok((mut stream, addr)) => {
                        println!("Accepted connection from {:?}", addr);
                        let client_fd = create_pool_fd(stream.as_raw_fd());
                        connections.accept_new_client(stream.as_raw_fd(), stream, client_fd);
                    }
                    Err(e) => {
                        println!("Accept error: {:?}", e);
                    }
                }
            } else if (pfd.revents & libc::POLLIN) != 0 {
                match connections.receive_data_and_respond(&pfd.fd.as_raw_fd()) {
                    None | Some(ConnectionStatus::Established) => {}
                    Some(ConnectionStatus::Closed) => {
                        let fd = pfd.fd.as_raw_fd();
                        println!("Client {fd} disconnected");
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
