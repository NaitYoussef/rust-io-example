use libc::{
    EPOLL_CTL_ADD, EPOLL_CTL_DEL, EPOLLET, EPOLLIN, c_int, epoll_create1 as unix_epoll_create1,
    epoll_ctl as unix_epoll_ctl, epoll_event, epoll_wait as unix_epoll_wait,
};
use rut_io_example::vfs::{ConnectionStatus, accept_new_client, close, receive_data_and_respond};
use std::io::{self};
use std::net::TcpListener;
use std::os::fd::{AsRawFd, RawFd};

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8000")?;
    listener.set_nonblocking(true)?;
    let epoll_fd = epoll_create1(0);
    if epoll_fd < 0 {
        return Err(io::Error::last_os_error());
    }

    let mut event = epoll_event {
        events: (EPOLLIN | EPOLLET) as u32,
        u64: listener.as_raw_fd() as u64,
    };

    epoll_ctl(epoll_fd, EPOLL_CTL_ADD, listener.as_raw_fd(), &mut event);

    let mut events = [epoll_event { events: 0, u64: 0 }; 1024];

    println!("Server epoll started !");
    loop {
        let number_of_events = epoll_wait(epoll_fd, events.as_mut_ptr(), 1024, -1);
        if number_of_events < 0 {
            return Err(io::Error::last_os_error());
        }
        for event in events.iter().take(number_of_events as usize) {
            let fd = event.u64 as RawFd;
            if fd == listener.as_raw_fd() {
                // Nouvelle connexion
                loop {
                    match listener.accept() {
                        Ok((stream, addr)) => {
                            let raw_fd = accept_new_client(stream, "Hello from epoll server\n")?;
                            println!("Accepted connection from {addr:?} fd {raw_fd}");
                            let mut ev = epoll_event {
                                events: (EPOLLIN | EPOLLET) as u32,
                                u64: raw_fd as u64,
                            };
                            epoll_ctl(epoll_fd, EPOLL_CTL_ADD, raw_fd, &mut ev);
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                            break;
                        }
                        Err(e) => {
                            println!("Accept error: {e:?}");
                            break;
                        }
                    }
                }
            } else {
                // Données à lire sur un client
                let status = receive_data_and_respond(fd, "Epoll server received your message !\n");

                match status {
                    Ok(ConnectionStatus::Established) => {}
                    Ok(ConnectionStatus::Closed) => {
                        println!("Client {fd} disconnected");
                        epoll_ctl(epoll_fd, EPOLL_CTL_DEL, fd, std::ptr::null_mut());
                        close(fd);
                    }
                    Err(e) => {
                        println!("Error reading from client: {} {:?}", fd, e);
                        epoll_ctl(epoll_fd, EPOLL_CTL_DEL, fd, std::ptr::null_mut());
                        close(fd);
                    }
                }
            }
        }
    }
}

fn epoll_wait(
    epfd: c_int,
    events: *mut crate::epoll_event,
    maxevents: c_int,
    timeout: c_int,
) -> isize {
    unsafe { unix_epoll_wait(epfd, events, maxevents, timeout) as isize }
}

fn epoll_ctl(epoll_fd: c_int, op: c_int, fd: c_int, event: *mut crate::epoll_event) {
    unsafe {
        unix_epoll_ctl(epoll_fd, op, fd, event);
    }
}

fn epoll_create1(flags: c_int) -> c_int {
    unsafe { unix_epoll_create1(flags) }
}
