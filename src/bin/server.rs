use std::{net::{TcpListener, TcpStream, SocketAddr}, io::{Read, BufReader}};

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:1883").unwrap();

    match listener.accept() {
        Ok((mut socket, addr)) => handle_connection(&mut socket, addr),
        Err(e) => println!("couldn't get client: {:?}", e),
    }

    Ok(())
}

fn handle_connection(socket: &mut TcpStream, addr: SocketAddr) {
    println!("new connection from {:?}", addr);

    let mut read = BufReader::new(socket);
    let mut buf: [u8; 1024] = [0; 1024];
    
    let num_bytes = read.read(&mut buf).unwrap();
    
    println!("finished reading {} bytes", num_bytes);
    
    let (msg, _) = buf.split_at(num_bytes);

    println!("RECEIVED: {:?}", msg);

    for b in msg {
        println!("{} :: {:x} :: {:08b}", b, b, b);
    }
}