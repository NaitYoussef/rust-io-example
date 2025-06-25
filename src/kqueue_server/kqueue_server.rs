use libc::pollfd;
use nix::sys::event::{EvFlags, EventFilter, EventFlag, FilterFlag, KEvent, Kqueue};
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
    let kq = nix::sys::event::kqueue().unwrap();
    let changelist = [KEvent::new(
        listener.as_raw_fd() as usize,
        EventFilter::EVFILT_READ,
        EvFlags::EV_ADD,
        FilterFlag::empty(),
        0,
        0,
    )];
    let mut clients = HashMap::new();
    unsafe { Kqueue::kevent(&kq, &changelist, &mut [], None).unwrap(); }

    loop {
        let mut events = [KEvent::new(0, EventFilter::EVFILT_READ, EventFlag::empty(), FilterFlag::empty(), 0, 0); 32];
        let nev = Kqueue::kevent(&kq, &[], &mut events, None).unwrap();
        for i in 0..nev {
            let event = events[i];
            if event.ident() == listener.as_raw_fd() as usize {
                // Nouvelle connexion
                match listener.accept() {
                    Ok((stream, addr)) => {
                        println!("Nouveau client : {}", addr);
                        stream.set_nonblocking(true)?;
                        let fd = stream.as_raw_fd();
                        clients.insert(fd, stream);
                        let ev = KEvent::new(
                            fd as usize,
                            EventFilter::EVFILT_READ,
                            EvFlags::EV_ADD,
                            FilterFlag::empty(),
                            0,
                            0,
                        );
                        Kqueue::kevent(&kq, &[ev], &mut [], None).unwrap();
                    }
                    Err(e) => {
                        eprintln!("Accept error: {:?}", e);
                    }
                }
            } else {
                let fd = event.ident() as i32;
                let mut to_remove = false;

                if let Some(stream) = clients.get_mut(&fd) {
                    let mut buf = [0u8; 1024];
                    match stream.read(&mut buf) {
                        Ok(0) => {
                            println!("Client déconnecté");
                            to_remove = true;
                        }
                        Ok(n) => {
                            println!("Reçu : {}", String::from_utf8_lossy(&buf[..n]));
                            stream.write_all(&buf[..n]).unwrap();
                        }
                        Err(e) => {
                            eprintln!("Read error: {:?}", e);
                            to_remove = true;
                        }
                    }
                }

                if to_remove {
                    clients.remove(&fd);
                    let ev = KEvent::new(
                        fd as usize,
                        EventFilter::EVFILT_READ,
                        EvFlags::EV_DELETE,
                        FilterFlag::empty(),
                        0,
                        0,
                    );
                    Kqueue::kevent(&kq, &[ev], &mut [], None)?;
                }
            }
        }
    }
}
