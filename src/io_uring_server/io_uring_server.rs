mod client_socket;

use crate::client_socket::{Buffer, Connection};
use io_uring::{opcode, types, IoUring};
use std::net::TcpListener;
use std::os::unix::io::AsRawFd;
use std::ptr;

const ACCEPT_FLAG: u64 = 1;

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8000")?;
    listener.set_nonblocking(true)?;
    println!("io_uring server started !");

    let mut ring = IoUring::new(256)?;

    let listener_fd = listener.as_raw_fd();

    submit_accept(&mut ring, listener_fd);

    let mut connections = Connection::new();
    loop {
        ring.submit_and_wait(1)?;
        let completions:Vec<(i32, u64)> = ring.completion().map(|completion| (completion.result(), completion.user_data())).collect();
        let mutable_ring = &mut ring;
        for cqe in completions {
            let result = cqe.0;
            let user_data = cqe.1;

            if user_data == ACCEPT_FLAG {
                // Reschedule a new accept
                submit_accept(mutable_ring, listener_fd);
                if result < 0 {
                    println!("Accept error");
                    continue;
                }
                println!("Accepted connection from {result:?}");
                connections.accept_new_client(result);
                let greetings = "Hello from io-uring server !\n";
                submit_write(mutable_ring, result, greetings, greetings.len());
            } else if user_data >= 2 && user_data <= 1000 {
                let fd = (user_data - 2) as i32;
                if result == 0 {
                    println!("Client {fd} disconnected");
                    connections.disconnect(fd);
                } else if result < 0 {
                    eprintln!(
                        "Error reading from client: {}",
                        std::io::Error::from_raw_os_error(-result)
                    );
                } else {
                    connections.receive_data(&fd, result as usize);
                    let greetings = "io_uring server received your message !\n";
                    submit_write(mutable_ring, fd, greetings, greetings.len());
                }
            } else if user_data >= 1000 {
                let client_fd = user_data - 1000;
                println!("Write finished on fd={}", client_fd);
                submit_read(
                    mutable_ring,
                    client_fd as i32,
                    connections.get_buffer(client_fd as i32),
                );
            }
        }
    }
}

fn get_all_completions(io_uring: IoUring){

}

fn submit_write(ring: &mut IoUring, fd: i32, buffer: &str, size: usize) {
    let ptr = buffer.as_bytes().as_ptr();

    let write_e = opcode::Write::new(types::Fd(fd), ptr, size as _)
        .build()
        .user_data((fd + 1000) as u64); // encode fd pour savoir à qui appartient l’opération

    unsafe {
        ring.submission().push(&write_e).expect("submission failed");
    }
}

// Soumet un accept
fn submit_accept(ring: &mut IoUring, listener_fd: i32) {
    let accept_e = opcode::Accept::new(types::Fd(listener_fd), ptr::null_mut(), ptr::null_mut())
        .build()
        .user_data(1); // On marque cet event comme étant un accept

    unsafe {
        ring.submission()
            .push(&accept_e)
            .expect("submission failed");
    }
}

// Soumet un read
fn submit_read(ring: &mut IoUring, client_fd: i32, buffer: Buffer) {
    let mut buf_lock = buffer.lock().unwrap();
    let read = opcode::Read::new(
        types::Fd(client_fd),
        buf_lock.as_mut_ptr(),
        buf_lock.len() as _,
    );
    let read_e = read.build().user_data((2 + client_fd) as u64);

    // Libère explicitement le lock avant le push
    drop(buf_lock);

    unsafe {
        ring.submission().push(&read_e).expect("submission failed");
    }
}
