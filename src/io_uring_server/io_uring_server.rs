use io_uring::{opcode, types, IoUring};
use std::net::TcpListener;
use std::os::unix::io::AsRawFd;
use std::ptr;
use std::time::SystemTime;

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8000")?;
    listener.set_nonblocking(true)?;
    println!("Serveur io-uring démarré !");

    let listener_fd = listener.as_raw_fd();
    let mut ring = IoUring::new(256).unwrap();

    // On soumet un accept dès le début
    submit_accept(&mut ring, listener_fd);

    loop {
        ring.submit_and_wait(1).unwrap();
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).expect("get millis error");
        println!("Something happend {}", now.as_millis());
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

                    need_accept = true;
                    // On resoumet un accept pour les prochains clients
                    //submit_accept(&mut ring, listener_fd);
                } else if user_data == 2 {
                    if result == 0 {
                        println!("Client déconnecté");
                        // Rien à faire ici, le client a fermé la connexion
                    } else if result < 0 {
                        eprintln!(
                            "Erreur lors de la lecture: {}",
                            std::io::Error::from_raw_os_error(-result)
                        );
                    } else {
                        let n = result as usize;
                        println!("Reçu {} octets", n);
                        // Ici tu pourrais écrire au client si tu veux

                        let fd = cqe.flags() as i32; // petite astuce pour récupérer le fd si tu l’avais stocké
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
            submit_read(&mut ring, fd);
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
fn submit_read(ring: &mut IoUring, client_fd: i32) {
    static mut BUF: [u8; 512] = [0; 512];

    let read_e = opcode::Read::new(
        types::Fd(client_fd),
        unsafe { BUF.as_mut_ptr() },
        BUF.len() as _,
    )
    .build()
    .user_data(2); // On marque cet event comme étant un read

    unsafe {
        ring.submission()
            .push(&read_e)
            .expect("soumission read échouée");
    }
}
