mod client_socket;

use crate::client_socket::{Connection, ConnectionStatus};
use nix::sys::event::{kqueue, EvFlags, EventFilter, FilterFlag, KEvent, Kqueue};
use nix::Error;
use std::io;
use std::io::prelude::*;
use std::net::TcpListener;
use std::os::fd::{AsRawFd, RawFd};

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8000")?;
    listener.set_nonblocking(true)?;
    let kq = kqueue()?;
    push_accept_event(&listener, &kq)?;
    let mut clients = Connection::new();
    println!("kqueue server started !");
    loop {
        let mut events = [KEvent::new(
            0,
            EventFilter::EVFILT_READ,
            EvFlags::empty(),
            FilterFlag::empty(),
            0,
            0,
        ); 32];
        let nev = Kqueue::kevent(&kq, &[], &mut events, None)?;
        for i in 0..nev {
            let event = events[i];
            if event.ident() == listener.as_raw_fd() as usize {
                match listener.accept() {
                    Ok((stream, addr)) => {
                        let fd = stream.as_raw_fd();
                        clients.accept_new_client(fd, stream);
                        println!("Accepted connection from {addr:?}");
                        push_read_event(&kq, &fd)?;
                    }
                    Err(e) => {
                        eprintln!("Accept error: {:?}", e);
                    }
                }
            } else {
                let fd = event.ident() as i32;
                match clients.receive_data_and_respond(&fd) {
                    None | Some(ConnectionStatus::Established) => {}
                    Some(ConnectionStatus::Closed(_)) => {
                        println!("Client {fd} disconnected");
                        push_remove_event(&kq, &fd)?;
                    }
                }
            }
        }
    }
}

fn push_read_event(kq: &Kqueue, fd: &RawFd) -> Result<(), Error> {
    let ev = new_event(&fd, EvFlags::EV_ADD);
    Kqueue::kevent(&kq, &ev, &mut [], None)?;
    Ok(())
}

fn push_accept_event(listener: &TcpListener, kq: &Kqueue) -> Result<(), Error> {
    let changelist = new_event(&listener.as_raw_fd(), EvFlags::EV_ADD);
    Kqueue::kevent(&kq, &changelist, &mut [], None)?;
    Ok(())
}

fn push_remove_event(kq: &Kqueue, fd: &i32) -> Result<(), Error> {
    let ev = new_event(&fd, EvFlags::EV_DELETE);
    Kqueue::kevent(&kq, &ev, &mut [], None)?;
    Ok(())
}

fn new_event(listener: &RawFd, flag: EvFlags) -> [KEvent; 1] {
    [KEvent::new(
        listener.as_raw_fd() as usize,
        EventFilter::EVFILT_READ,
        flag,
        FilterFlag::empty(),
        0,
        0,
    )]
}
