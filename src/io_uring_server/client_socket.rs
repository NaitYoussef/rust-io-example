use std::collections::HashMap;
use std::os::fd::RawFd;

pub struct Connection {
    buffers: HashMap<i32, Vec<u8>>,
}

impl Connection {
    pub fn new() -> Self {
        Connection {
            buffers: HashMap::new(),
        }
    }

    pub fn disconnect(&mut self, fd: RawFd) {
        self.buffers.remove(&fd);
    }

    pub fn get_buffer(&self, fd: i32) -> Vec<u8> {
        self.buffers.get(&fd).unwrap().clone()
    }

    pub fn accept_new_client(&mut self, fd: RawFd) {
        self.buffers.insert(fd, vec![0u8; 512]);
    }

    pub fn receive_data(&mut self, fd: &RawFd, n: usize) {
        let buf = self.buffers.get(fd).unwrap();
        println!(
            "Received {} from {}",
            String::from_utf8_lossy(&buf[..n]).replace('\n', ""),
            fd
        );
    }
}
