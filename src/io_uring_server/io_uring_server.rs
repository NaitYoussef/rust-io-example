mod client_socket;

use crate::client_socket::Connection;
use io_uring::{IoUring, opcode, types};
use std::net::TcpListener;
use std::os::unix::io::AsRawFd;
use std::ptr;

const DISCONNECTED: i32 = 0;

const OPERATION_ACCEPT: i32 = 1;
const OPERATION_READ: i32 = 2;
const OPERATION_WRITE: i32 = 3;

enum OpType {
    Accept,
    Read(i32),
    Write(i32),
}

// use a tryFrom implementation to convert from u64 to OpType, externalize OpType in specific module
impl From<u64> for OpType {
    fn from(value: u64) -> Self {
        let fd: i32 = (value & 0xFFFF_FFFF) as i32;
        let operation: i32 = (value >> 32) as i32;
        if operation == OPERATION_ACCEPT {
            OpType::Accept
        } else if operation == OPERATION_READ {
            OpType::Read(fd)
        } else if operation == OPERATION_WRITE {
            OpType::Write(fd)
        } else {
            panic!("Unknown operation type for fd: {fd}");
        }
    }
}

impl From<OpType> for u64 {
    fn from(value: OpType) -> Self {
        match value {
            OpType::Accept => 1u64 << 32,
            OpType::Read(fd) => (2u64 << 32) | (fd as u32 as u64),
            OpType::Write(fd) => (3u64 << 32) | (fd as u32 as u64),
        }
    }
}

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
        let completions: Vec<(i32, u64)> = ring
            .completion()
            .map(|completion| (completion.result(), completion.user_data()))
            .collect();
        for cqe in completions {
            let result = cqe.0;
            let user_data = cqe.1;

            if result < 0 {
                eprintln!(
                    "error in completion: {}",
                    std::io::Error::from_raw_os_error(-result)
                );
                continue;
            }

            let op_type: OpType = user_data.into();

            match op_type {
                OpType::Accept => {
                    submit_accept(&mut ring, listener_fd);
                    println!("Accepted connection from {result:?} fd {result}");
                    connections.accept_new_client(result);
                    let greetings = "Hello from io-uring server !\n";
                    submit_write(&mut ring, result, greetings, greetings.len());
                }
                OpType::Read(fd) => {
                    if result == DISCONNECTED {
                        println!("Client {fd} disconnected");
                        connections.disconnect(fd);
                    } else {
                        connections.receive_data(&fd, result as usize);
                        let greetings = "io_uring server received your message !\n";
                        submit_write(&mut ring, fd, greetings, greetings.len());
                    }
                }
                OpType::Write(fd) => {
                    println!("Write finished on fd={fd}");
                    submit_read(&mut ring, fd, connections.get_buffer(fd));
                }
            }
        }
    }
}

fn submit_write(ring: &mut IoUring, fd: i32, buffer: &str, size: usize) {
    let ptr = buffer.as_bytes().as_ptr();
    let op_type = OpType::Write(fd);
    let write_e = opcode::Write::new(types::Fd(fd), ptr, size as _)
        .build()
        .user_data(u64::from(op_type)); // encode fd pour savoir à qui appartient l’opération

    unsafe {
        ring.submission().push(&write_e).expect("submission failed");
    }
}

// Soumet un accept
fn submit_accept(ring: &mut IoUring, listener_fd: i32) {
    let accept_e = opcode::Accept::new(types::Fd(listener_fd), ptr::null_mut(), ptr::null_mut())
        .build()
        .user_data(u64::from(OpType::Accept)); // On marque cet event comme étant un accept

    unsafe {
        ring.submission()
            .push(&accept_e)
            .expect("submission failed");
    }
}

// Soumet un read
fn submit_read(ring: &mut IoUring, client_fd: i32, buffer: Vec<u8>) {
    let mut buffer = buffer.clone();
    let op_type = OpType::Read(client_fd);
    let read = opcode::Read::new(types::Fd(client_fd), buffer.as_mut_ptr(), buffer.len() as _)
        .build()
        .user_data(u64::from(op_type));

    unsafe {
        ring.submission().push(&read).expect("submission failed");
    }
}
