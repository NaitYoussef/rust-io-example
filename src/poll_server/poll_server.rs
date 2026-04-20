use libc::pollfd;
use rut_io_example::vfs::{ConnectionStatus, accept_new_client, close, receive_data_and_respond};
use std::io;
use std::net::TcpListener;
use std::os::fd::{AsRawFd, RawFd};

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8000")?;
    listener.set_nonblocking(true)?;
    let poll_fd = create_pool_fd(listener.as_raw_fd());

    // liste des fds stockés dans le programme
    let mut poll_fds = vec![poll_fd];

    println!("Poll server started !");
    loop {
        poll(&mut poll_fds);
        let cloned_fds = poll_fds.clone();

        let filtered_fds: Vec<pollfd> = cloned_fds
            .into_iter()
            .filter(|pfd| pfd.revents & libc::POLLIN != 0)
            .collect();

        for pfd in &filtered_fds {
            if pfd.fd == listener.as_raw_fd() {
                loop {
                    match listener.accept() {
                        Ok((stream, addr)) => {
                            println!(
                                "Accepted connection from {:?} fd {}",
                                addr,
                                stream.as_raw_fd()
                            );
                            let raw_fd = accept_new_client(stream, "Hello from poll server\n")?;
                            let client_fd = create_pool_fd(raw_fd);
                            poll_fds.push(client_fd);
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
                        Err(e) => {
                            println!("Accept error: {e:?}");
                            break;
                        }
                    }
                }
            } else {
                let fd = pfd.fd;
                let status = receive_data_and_respond(fd, "Poll server received your message !\n");
                match status {
                    Ok(ConnectionStatus::Established) => {}
                    Ok(ConnectionStatus::Closed) => {
                        println!("Client {fd} disconnected");
                        poll_fds.retain(|poll_fd| poll_fd.fd != fd);
                        close(fd);
                    }
                    Err(e) => {
                        println!("Error reading from client: {} {:?}", fd, e);
                        poll_fds.retain(|poll_fd| poll_fd.fd != fd);
                        close(fd);
                    }
                }
            }
        }
    }
}

fn poll(fds: &mut Vec<pollfd>) {
    unsafe {
        libc::poll(
            fds.as_mut_ptr(),
            fds.len() as libc::nfds_t,
            -1 as libc::c_int,
        );
    }
}

fn create_pool_fd(fd: RawFd) -> pollfd {
    pollfd {
        fd,
        events: libc::POLLIN,
        revents: 0,
    }
}
