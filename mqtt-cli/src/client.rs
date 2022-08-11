use std::{net::TcpStream, io::{Write, Read}};

use mqtt::{error::MqttError, packet::{Connect, Connack, Publish, Disconnect, Puback, PacketType, Pubrec, Pubrel, Pubcomp, ConnackProperties, Subscribe}, types::{QoS, ReasonCode}};

use crate::{Session, CmdResult};

pub struct Client {
    session: Session,
    client_id: String,
    packet_id: Option<u16>,
    connected: bool,
    stream: TcpStream,
}

impl Client {

    pub fn connect(session: Session) -> Result<Self, MqttError> {
        let addr = session.addr();
        println!("Connecting to {:?}", addr);

        let stream = TcpStream::connect(&addr).unwrap_or_else(|e| {
            panic!("Error establishing connection to server: {:?}", e)
        });

        let mut client = Client {
            session,
            client_id: String::new(),
            packet_id: None,
            connected: false,
            stream,
        };
        let connect = Connect::default();
        println!("CONNECT: {:?}", connect);

        client.send(connect)?;
        let connack_bytes = client.receive()?;
        let connack = Connack::try_from(&connack_bytes[..])?;
        
        println!("CONNACK: {:?}", connack);
        
        client.connected = true;

        if let Some(ConnackProperties { assigned_client_identifier, .. }) = connack.properties {
            if let Some(s) = assigned_client_identifier {
                client.client_id = s;
            }
        }

        Ok(client)
    }

    pub fn publish(&mut self, packet: Publish) -> CmdResult {
        let qos = packet.qos_level.clone();
        self.packet_id = packet.packet_identifier;
        println!("PUBLISH: {:?}", packet);
        self.send(packet)?;
        match qos {
            QoS::AtMostOnce => Ok(()),
            els => self.handle_pub_qos(els),
        }
    }

    pub fn subscribe(&mut self, packet: Subscribe) -> CmdResult {
        println!("SUBSCRIBE: {:?}", packet);
        self.send(packet)?;

        let response = self.receive()?;
        match PacketType::try_from(response[0])? {
            PacketType::SUBACK => {
                let suback = mqtt::packet::Suback::try_from(&response[..])?;
                println!("SUBACK: {:?}", suback);
                //self.listen();
                Ok(())
            },
            PacketType::DISCONNECT => {
                let disconnect = Disconnect::try_from(&response[..])?;
                println!("DISCONNECT: {:?}", disconnect);
                self.connected = false;
                Err(MqttError::Message(format!("Server disconnected after SUBSCRIBE with reason code {:?}", disconnect.reason_code)))
            },
            _=> {
                Err(MqttError::ProtocolError(format!("Unexpected response message: {:?}", response)))
            },
        }
    }

    /// clones the `TcpStream` of this client and spawns a new thread to listen to incoming messages.
    pub fn listen(&mut self) {
        // FIXME don't just unwrap!
        let mut stream = self.stream.try_clone().unwrap();
        std::thread::spawn(move || {
            loop {
                if let Ok(rec) = receive_raw(&mut stream) {
                    if rec.len() > 0 {
                        match PacketType::try_from(rec[0]).unwrap() {
                            PacketType::PUBLISH => {
                                let publ = Publish::try_from(&rec[..]).unwrap();
                                println!("Received PUBLISH: {:?}", publ)
                            },
                            els => println!("Received unexepcted packet {:?}: {:?}", els, rec),
                        }
                    }
                }
            }
        });
    }

    pub fn disconnect(&mut self) -> CmdResult {
        if !self.connected {
            return Ok(())
        }

        let disconnect = Disconnect::default();
        println!("DISCONNECT: {:?}", disconnect);
        self.send(disconnect)?;
        match self.stream.shutdown(std::net::Shutdown::Both) {
            Ok(_) => Ok(()),
            Err(e) => Err(MqttError::Message(format!("Error closing stream: {:?}", e))),
        }
        
    }

    fn handle_pub_qos(&mut self, qos: QoS) -> CmdResult {
        let response = self.receive()?;
        match PacketType::try_from(response[0])? {
            PacketType::DISCONNECT => {
                let disconnect = Disconnect::try_from(&response[..])?;
                println!("DISCONNECT: {:?}", disconnect);
                self.connected = false;
                Err(MqttError::Message(format!("Server disconnected after PUBLISH with reason code {:?}", disconnect.reason_code)))
            },
            PacketType::PUBACK => {
                println!("PUBACK {:?}", Puback::try_from(&response[..]));
                Ok(())
            },
            PacketType::PUBREC => {
                let pubrec = Pubrec::try_from(&response[..])?;
                println!("PUBREC: {:?}", pubrec);
                let reason_code = match Some(pubrec.packet_identifier) == self.packet_id {
                    true => ReasonCode::Success,
                    false => ReasonCode::PacketIdentifierNotFound,
                };
                let pubrel = Pubrel::new(pubrec.packet_identifier, reason_code)?;
                self.send(pubrel)?;
                self.handle_pub_qos(qos)
            },
            PacketType::PUBREL => {
                let pubrel = Pubrel::try_from(&response[..])?;
                println!("PUBREL: {:?}", pubrel);
                let reason_code = match Some(pubrel.packet_identifier) == self.packet_id {
                    true => ReasonCode::Success,
                    false => ReasonCode::PacketIdentifierNotFound,
                };
                let pubcomp = Pubcomp::new(pubrel.packet_identifier, reason_code)?;
                self.send(pubcomp)
            },
            PacketType::PUBCOMP => {
                let pubcomp = Pubcomp::try_from(&response[..])?;
                println!("PUBCOMP: {:?}", pubcomp);
                Ok(())
            }
            _=> {
                println!("RESPONSE_NOT_YET_IMPLEMENTED: {:?}", response);
                Err(MqttError::ProtocolError(format!("Unexpected response message: {:?}", response)))
            },
        }
    }

    fn send<P: Into<Vec<u8>>>(&mut self, packet: P) -> CmdResult {
        let binary = packet.into();
    
        self.session.debug(format!("Sending {} bytes to server", binary.len()));
        self.session.debug(format!("{:?}", binary));
    
        if let Err(e) = self.stream.write_all(&binary[..]) {
            return Err(MqttError::Message(format!("Error sending CONNECT: {:?}", e)))
        }
    
        Ok(())
    }
    
    fn receive(&mut self) -> Result<Vec<u8>, MqttError> {
        let mut buff: [u8; 4048] = [0; 4048];
        match self.stream.read(&mut buff) {
            Ok(num_bytes) => {
                self.session.debug(format!("Read {} bytes from server", num_bytes));
                let mut result: Vec<u8> = Vec::with_capacity(num_bytes);
                result.extend_from_slice(&buff[..num_bytes]);
                self.session.debug(format!("{:?}", result));
                return Ok(result)
            },
            Err(e) => return Err(MqttError::Message(format!("Error reading from stream: {:?}", e))),
        }
    }
}

/// need this function so there's no pointers to or ownership issues with the `Client` itself.
fn receive_raw<R>(stream: &mut R) -> Result<Vec<u8>, MqttError> 
where
    R: Read
{
    const BUFFER_SIZE: usize = 4096;
    let mut buff: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
    match stream.read(&mut buff) {
        Ok(num_bytes) => {
            let mut result: Vec<u8> = Vec::with_capacity(num_bytes);
            result.extend_from_slice(&buff[..num_bytes]);

            if num_bytes == BUFFER_SIZE {
                result.extend_from_slice(&receive_raw(stream)?);
            }

            return Ok(result)
        },
        Err(e) => return Err(MqttError::Message(format!("Error reading from stream: {:?}", e))),
    }
}

