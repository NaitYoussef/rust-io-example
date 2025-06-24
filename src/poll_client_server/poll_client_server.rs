use std::io;
use std::io::prelude::*;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::os::fd::AsRawFd;
use std::sync::Mutex;
use std::task::Poll;
use std::thread;
use std::time::Duration;

static POLL_FDS: Mutex<Vec<libc::pollfd>> = Mutex::new(Vec::new());

unsafe fn poll(
    fd: &impl AsRawFd,
    events: libc::c_short,
) {
    let mut pollfd = libc::pollfd {
        fd: fd.as_raw_fd(),
        events,
        revents: 0,
    };
    libc::poll(
        &mut pollfd,
        1 as libc::nfds_t,
        10 as libc::c_int,
    );
}

fn register_pollfd(
    fd: &impl AsRawFd,
    events: libc::c_short,
) {
    let mut poll_fds = POLL_FDS.lock().unwrap();
    poll_fds.push(libc::pollfd {
        fd: fd.as_raw_fd(),
        events,
        revents: 0,
    });
}

async fn accept(
    listener: &mut TcpListener,
) -> io::Result<(TcpStream, SocketAddr)> {
    std::future::poll_fn(|context| match listener.accept() {
        Ok((stream, addr)) => {
            stream.set_nonblocking(true)?;
            Poll::Ready(Ok((stream, addr)))
        }
        Err(e) if e.kind() == io::ErrorKind::WouldBlock => unsafe {
            register_pollfd(listener, libc::POLLIN);
            Poll::Pending
        }
        Err(e) => Poll::Ready(Err(e)),
    })
        .await
}

fn server_main(mut listener: TcpListener) -> io::Result<()> {
    let mut n = 1;
    loop {
        let (mut socket, _) = accept(&mut listener)?;
        // Using format! instead of write! avoids breaking up lines across multiple writes. This is
        // easier than doing line buffering on the client side.
        let start_msg = format!("start {n}\n");
        socket.write_all(start_msg.as_bytes())?;
        thread::sleep(Duration::from_secs(1));
        let end_msg = format!("end {n}\n");
        socket.write_all(end_msg.as_bytes())?;
        n += 1;
    }
}

// TODO add client durations
fn client_main() -> io::Result<()> {
    let mut socket = TcpStream::connect("localhost:8000")?;
    io::copy(&mut socket, &mut io::stdout())?;
    Ok(())
}

fn main() -> io::Result<()> {
    // Open the listener first, to avoid racing against the server thread.
    let listener = TcpListener::bind("0.0.0.0:8000")?;
    listener.set_nonblocking(true)?;
    // Start the server on a background thread.
    thread::spawn(|| server_main(listener).unwrap());
    // Run ten clients on ten different threads.
    let mut client_handles = Vec::new();
    for _ in 1..=10 {
        client_handles.push(thread::spawn(client_main));
    }
    for handle in client_handles {
        handle.join().unwrap()?;
    }
    Ok(())
}
