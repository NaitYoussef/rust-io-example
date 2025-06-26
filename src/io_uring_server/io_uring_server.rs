use std::collections::HashMap;
use io_uring::{opcode, types, IoUring};
use std::net::TcpListener;
use std::os::unix::io::AsRawFd;
use std::ptr;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8000")?;
    listener.set_nonblocking(true)?;
    println!("Serveur io-uring démarré !");

    let listener_fd = listener.as_raw_fd();
    let mut ring = IoUring::new(256).unwrap();

    // On soumet un accept dès le début
    submit_accept(&mut ring, listener_fd);
    let buffers: Arc<Mutex<HashMap<i32, Arc<Mutex<Vec<u8>>>>>> = Arc::new(Mutex::new(HashMap::new()));

    loop {
        ring.submit_and_wait(1).unwrap();
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).expect("get millis error");
        let mut need_accept = false;
        let mut to_read = vec![];
        {
            let mut completions = ring.completion();
            while let Some(cqe) = completions.next() {
                let result = cqe.result();
                let user_data = cqe.user_data();

                if user_data == 1 {
                    // Accept complété
                    if result < 0 {
                        eprintln!(
                            "Erreur lors du accept: {}",
                            std::io::Error::from_raw_os_error(-result)
                        );
                        // On resoumet un accept
                        // submit_accept(&mut ring, listener_fd);
                        need_accept = true;
                        continue;
                    }

                    let client_fd = result;
                    println!("Nouveau client connecté: fd={}", client_fd);

                    // Soumettre une lecture sur le client
                    to_read.push(client_fd);
                    // submit_read(&mut ring, client_fd);
                    let buf = Arc::new(Mutex::new(vec![0u8; 512]));
                    buffers.lock().unwrap().insert(client_fd, buf.clone());
                    need_accept = true;
                    // On resoumet un accept pour les prochains clients
                    //submit_accept(&mut ring, listener_fd);
                } else if user_data >= 2 {
                    if result == 0 {
                        println!("Client déconnecté");
                        // Rien à faire ici, le client a fermé la connexion
                        buffers.lock().unwrap().remove(&(result as i32));
                    } else if result < 0 {
                        eprintln!(
                            "Erreur lors de la lecture: {}",
                            std::io::Error::from_raw_os_error(-result)
                        );
                    } else {
                        let n = result as usize;
                        let fd = (user_data - 2) as i32; // petite astuce pour récupérer le fd si tu l’avais stocké
                        let guard = buffers.lock().unwrap();
                        let buf_mutex = guard.get(&fd).unwrap();
                        let buf = buf_mutex.lock().unwrap();
                        println!("Received data is {}", String::from_utf8_lossy(&buf[..n]));
                        // Ici tu pourrais écrire au client si tu veux

                        to_read.push(fd);
                        //submit_read(&mut ring, fd);  // relancer une lecture sur ce fd
                    }
                }
            }
        }
        // Après avoir fini d’itérer :
        if need_accept {
            submit_accept(&mut ring, listener_fd);
        }
        for fd in to_read {
            if let Some(buf) = buffers.lock().unwrap().get(&fd) {
                submit_read(&mut ring, fd, buf.clone());
            }
        }
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
            .expect("soumission accept échouée");
    }
}

// Soumet un read
fn submit_read(ring: &mut IoUring, client_fd: i32, buffer: Arc<Mutex<Vec<u8>>>) {
    let mut buf_lock = buffer.lock().unwrap();
    let read = opcode::Read::new(
        types::Fd(client_fd),
        buf_lock.as_mut_ptr(),
        buf_lock.len() as _,
    );
    let read_e = read
        .build()
        .user_data((2 + client_fd) as u64);

    // Libère explicitement le lock avant le push
    drop(buf_lock);

    unsafe {
        ring.submission().push(&read_e).expect("soumission read échouée");
    }
}