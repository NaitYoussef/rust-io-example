use std::collections::HashMap;
use std::os::fd::RawFd;
use std::sync::{Arc, Mutex};

pub type Buffer = Arc<Mutex<Vec<u8>>>;
pub struct Connection {
    buffers: Arc<Mutex<HashMap<i32, Buffer>>>
}

pub enum ConnectionStatus {
    Established,
    Closed,
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

    pub fn get_buffer(&self, fd:i32) -> Buffer {
        self.buffers.lock().unwrap().get(&fd).unwrap().clone()
    }

    pub fn accept_new_client(&mut self, fd: RawFd) {
        let buf = Arc::new(Mutex::new(vec![0u8; 512]));
        self.buffers.lock().unwrap().insert(fd, buf.clone());
    }

    pub fn receive_data(&mut self, fd: &RawFd, n: usize) {
        let guard = self.buffers.lock().unwrap();
        let buf_mutex = guard.get(&fd).unwrap();
        let buf = buf_mutex.lock().unwrap();
        println!("Received {}", String::from_utf8_lossy(&buf[..n]));
    }
}
