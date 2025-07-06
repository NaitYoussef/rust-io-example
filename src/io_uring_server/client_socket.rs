use std::collections::HashMap;
use std::os::fd::RawFd;
use std::sync::{Arc, Mutex};

pub struct Connection {
    buffers: Arc<Mutex<HashMap<i32, Vec<u8>>>>
}

impl Connection {
    pub fn new() -> Self {
        Connection {
            buffers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn disconnect(&mut self, fd: RawFd) {
        self.buffers.lock().unwrap().remove(&fd);
    }

    pub fn get_buffer(&self, fd:i32) -> Vec<u8> {
        self.buffers.lock().unwrap().get(&fd).unwrap().clone()
    }

    pub fn accept_new_client(&mut self, fd: RawFd) {
        self.buffers.lock().unwrap().insert(fd, vec![0u8; 512]);
    }

    pub fn receive_data(&mut self, fd: &RawFd, n: usize) {
        let guard = self.buffers.lock().unwrap();
        let buf = guard.get(fd).unwrap();
        println!("Received {} from {}", String::from_utf8_lossy(&buf[..n]).replace('\n', ""), fd);
    }
}
