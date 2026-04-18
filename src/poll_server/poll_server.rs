use libc::pollfd;
use rut_io_example::vfs::{ClientSocket, ConnectionStatus};
use std::collections::HashMap;
use std::io;
use std::net::TcpListener;
use std::os::fd::{AsRawFd, RawFd};

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8000")?;
    listener.set_nonblocking(true)?;
    let poll_fd = create_pool_fd(listener.as_raw_fd());

    // liste des fds stockés dans le programme
    let mut poll_fds = vec![poll_fd];

    let mut clients_socket: HashMap<RawFd, ClientSocket> = HashMap::new();
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
                match listener.accept() {
                    Ok((stream, addr)) => {
                        println!("Accepted connection from {:?} fd {}", addr, stream.as_raw_fd());
                        let socket = ClientSocket::accept_new_client(stream)?;
                        let fd = socket.fd();
                        let client_fd = create_pool_fd(fd);
                        poll_fds.push(client_fd);
                        clients_socket.insert(fd, socket);
                    }
                    Err(e) => {
                        println!("Accept error: {e:?}");
                    }
                }
            } else {
                let fd = pfd.fd.as_raw_fd();
                let status = if let Some(client) = clients_socket.get_mut(&fd) {
                    client.receive_data_and_respond()
                } else {
                    continue;
                };

                match status {
                    Ok(ConnectionStatus::Established) => {}
                    Ok(ConnectionStatus::Closed) => {
                        println!("Client {fd} disconnected");
                        remove_client(&mut clients_socket, &mut poll_fds, fd);
                    }
                    Err(e) => {
                        println!("Error reading from client: {} {:?}", fd, e);
                        remove_client(&mut clients_socket, &mut poll_fds, fd);
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

fn remove_client(
    clients_socket: &mut HashMap<RawFd, ClientSocket>,
    poll_fds: &mut Vec<pollfd>,
    fd: RawFd,
) {
    clients_socket.remove(&fd);
    poll_fds.retain(|poll_fd| poll_fd.fd != fd);
}
