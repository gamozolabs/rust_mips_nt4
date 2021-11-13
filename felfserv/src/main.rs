use std::io;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::net::{TcpStream, TcpListener};

fn handle_client(mut sock: TcpStream, file: PathBuf) -> io::Result<()> {
    let payload = std::fs::read(file)?;

    print!("\n\nServing {} bytes to {:?}\n---------------------------------\n",
        payload.len(), sock.peer_addr()?);

    // Write the header
    sock.write_all(&(payload.len() as u32).to_le_bytes())?;
    sock.write_all(&payload)?;

    // Read data from user
    loop {
        let mut bytes = [0u8; 1024];
        let bread = sock.read(&mut bytes)?;
        if bread == 0 {
            return Ok(());
        }

        print!("{}", std::str::from_utf8(&bytes[..bread]).unwrap());
    }
}

fn main() -> io::Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() != 3 {
        print!("usage: felfserv <bind ip:port> <file>\n");
        return Ok(());
    }

    let listener = TcpListener::bind(&args[1])?;
    for stream in listener.incoming() {
        let path = PathBuf::from(&args[2]);
        std::thread::spawn(move || {
            handle_client(stream.unwrap(), path).unwrap()
        });
    }

    Ok(())
}

