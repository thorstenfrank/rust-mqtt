use std::{net::{TcpListener, TcpStream, SocketAddr}, io::Read};

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

    let mut buf: Vec<u8> = Vec::with_capacity(1024);
    match socket.read_to_end(&mut buf) {
        Ok(len) => {
            println!("finished reading {} bytes", len);
            for b in buf {
                println!("{} :: {:x} :: {:08b}", b, b, b);
            }
        },
        Err(e) => println!("error reading from socket: {:?}", e),
    }
}