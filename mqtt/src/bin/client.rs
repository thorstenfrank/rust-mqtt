use std::{net::TcpStream, io::{BufReader, Write, Read}};

use mqtt::packet::{Connect, Connack};

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:1883")?;

    let conn = Connect::with_client_id_str("my-rust-mqtt-client-0").unwrap();
    println!("sending CONNECT: {:?}", conn);

    let buf: Vec<u8> = conn.into();
    // a working example from mosquitto
    //vec![16, 22, 0, 4, 77, 81, 84, 84, 5, 2, 0, 60, 3, 33, 0, 1, 0, 6, 87, 85, 80, 80, 68, 73];
    
    println!("{:?}", buf);
    match stream.write_all(buf.as_slice()) {
        Err(e) => println!("error: {:?}", e),
        Ok(_) => println!("ok!"),
    };

    let mut reader = BufReader::new(stream);
    let mut buf = [0; 1024];

    match reader.read(&mut buf) {
        Ok(len) => {
            println!("read {} bytes from server", len);
            print_bin(&buf[..len]);
            let connack = Connack::try_from(&buf[..]).unwrap();
            println!("{:?}", connack)
        },
        Err(e) => println!("error receiving CONNACK: {:?}", e), 
    };

    Ok(())
}

fn print_bin(bin: &[u8]) {
    for b in bin {
        println!("{} :: {:x} :: {:08b}", b, b, b);
    }
}