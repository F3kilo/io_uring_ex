use io_uring::{opcode, types, IoUring};
use std::net::TcpStream;
use std::os::unix::io::AsRawFd;
use std::time::Duration;
use std::{io, thread};

fn main() -> io::Result<()> {
    let stream = TcpStream::connect("127.0.0.1:55331").unwrap();
    stream.set_nonblocking(true).unwrap();
    let fd = stream.as_raw_fd();

    let mut ring = IoUring::new(8)?;

    let mut buf = vec![0; 1];

    loop {
        let recv_e = opcode::Recv::new(types::Fd(fd.as_raw_fd()), buf.as_mut_ptr(), buf.len() as _)
            .build()
            .user_data(42);

        unsafe {
            ring.submission()
                .push(&recv_e)
                .expect("submission queue is full");
        }

        ring.submit()?;

        match ring.completion().next() {
            Some(entry) => {
                println!();
                println!("************ New entry ************");
                println!("User data: {}", entry.user_data());
                println!("Red len: {}", entry.result());
                println!("Data red: {:?}", buf[0]);
                println!();
                if entry.result() == 0 {
                    break;
                }
            }
            None => {
                println!("no entries in completion queue");
            }
        }

        thread::sleep(Duration::from_millis(300));
    };

    Ok(())
}
