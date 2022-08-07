use std::{net::TcpStream, io::{Write, Read}};

use mqtt::{error::MqttError, packet::{Connect, Connack, Publish, Disconnect, Puback, PacketType, Pubrec, Pubrel, Pubcomp, ConnackProperties, Subscribe}, types::{QoS, ReasonCode}};

use crate::Session;

pub struct Client {
    session: Session,
    client_id: String,
    packet_id: Option<u16>,
    connected: bool,
    stream: TcpStream,
}

impl Client {

    //const DEFAULT_PORT: u16 = 1883;
    //const DEFAULT_PORT_SECURE: u16 = 8883;

    pub fn connect(session: Session) -> Result<Self, MqttError> {
        println!("Connecting to {:?}", session.addr);

        let stream = TcpStream::connect(&session.addr).unwrap_or_else(|e| {
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

    pub fn publish(&mut self, packet: Publish) -> Result<(), MqttError> {
        let qos = packet.qos_level.clone();
        self.packet_id = packet.packet_identifier;
        println!("PUBLISH: {:?}", packet);
        self.send(packet)?;
        match qos {
            QoS::AtMostOnce => Ok(()),
            els => self.handle_pub_qos(els),
        }
    }

    pub fn subscribe(&mut self, packet: Subscribe) -> Result<(), MqttError> {
        println!("SUBSCRIBE: {:?}", packet);
        self.send(packet)?;

        let response = self.receive()?;
        match PacketType::try_from(response[0])? {
            PacketType::SUBACK => {
                let suback = mqtt::packet::Suback::try_from(&response[..])?;
                println!("SUBACK: {:?}", suback);
                self.listen();
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

    fn listen(&mut self) {
        loop {
            if let Ok(rec) = self.receive() {
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
    }

    pub fn disconnect(&mut self) -> Result<(), MqttError> {
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

    fn handle_pub_qos(&mut self, qos: QoS) -> Result<(), MqttError> {
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

    fn send<P: Into<Vec<u8>>>(&mut self, packet: P) -> Result<(), MqttError> {
        let binary = packet.into();
    
        if self.session.debug {
            println!("Sending {} bytes to server", binary.len());
            println!("{:?}", binary);
        }
    
        if let Err(e) = self.stream.write_all(&binary[..]) {
            return Err(MqttError::Message(format!("Error sending CONNECT: {:?}", e)))
        }
    
        Ok(())
    }
    
    fn receive(&mut self) -> Result<Vec<u8>, MqttError> {
        let mut buff: [u8; 4048] = [0; 4048];
        match self.stream.read(&mut buff) {
            Ok(num_bytes) => {
                if self.session.debug {
                    println!("Read {} bytes from server", num_bytes);
                }
                let mut result: Vec<u8> = Vec::with_capacity(num_bytes);
                result.extend_from_slice(&buff[..num_bytes]);
                if self.session.debug {
                    println!("{:?}", result);
                }
                
                return Ok(result)
            },
            Err(e) => return Err(MqttError::Message(format!("Error reading from stream: {:?}", e))),
        }
    }
}


