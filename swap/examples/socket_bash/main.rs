use std::error::Error;
use std::io::{self, ErrorKind};
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, Interest};
use tokio::net::{UnixListener, UnixStream};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let ex_sock_path = "/tmp/example.sock";
    let _ = std::fs::remove_file(ex_sock_path);

    let rx = UnixListener::bind(ex_sock_path)?;

    let tx = UnixStream::connect(ex_sock_path).await?;

    let mut inbound: Option<UnixStream> = None;

    let ready = tx.ready(Interest::READABLE | Interest::WRITABLE).await?;

    match rx.accept().await {
        Ok((stream, _addr)) => {
            println!("new client!");
            let _ = inbound.insert(stream);
            ()
        }
        Err(_e) => { /* connection failed */ }
    }

    let mut i = 0;
    loop {
        match inbound {
            None => (),
            Some(ref mut x) => {
                let mut data = vec![0; 255];
                match x.try_read(&mut data) {
                    Ok(0) => (),
                    Ok(n) => {
                        // println!("read {} bytes", n);
                        println!("read ({n}): {}", String::from_utf8(data).unwrap());
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => (),
                    Err(e) => return Err(e.into()),
                }
            }
        };

        if ready.is_writable() {
            match tx.try_write(format!("hello world {i}").as_bytes()) {
                Ok(n) => {
                    println!("write {} bytes", n);
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        }
        i += 1;
    }
}
