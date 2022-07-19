use std::collections::HashMap;

use crate::{types::*, error::MqttError, packet::calculate_and_insert_length};

use super::{MqttControlPacket, PacketType, push_be_u32, push_be_u16};

const FIRST_BYTE: u8 = 0b00100000;
/// A `CONNACK` MQTT control packet.
#[derive(Debug)]
pub struct ConnackPacket {

    /// Whether this connect/connack exchange resumes an existing session or starts a new one.
    pub session_present: bool,

    /// Indicates whether the connection attempt was successful, and if not why.
    /// [Anything above 0x80 is an error](crate::types::ReasonCode::is_err()).
    pub reason_code: ReasonCode,

    /// Optional properties sent by the server.
    pub properties: Option<ConnackProperties>,
}

/// Sums up all properties a server may send.
#[derive(Debug)]
pub struct ConnackProperties {

    /// Server override for an interval requested by the client 
    /// [with the CONNECT properties](super::ConnectProperties::session_expiry_interval).
    pub session_expiry_interval: Option<u32>,

    /// Limits on concurrent QoS 1 and 2 messages.
    pub receive_maximum: Option<u16>,

    /// Limits the maximum quality of service level the server supports.
    /// Defaults to [QoS 2](crate::types::QoS::ExactlyOnce).
    pub maximum_qos: Option<QoS>,

    /// Whether or not the server supports message retention for will messages.
    pub retain_available: bool,

    /// Maximum size in number of bytes the server is willing to accept.
    pub max_packet_size: Option<u32>,

    /// Server-issued in cases where the client does not specify its own ID with the `CONNECT` packet.
    pub assigned_client_id: Option<String>,

    /// Maximum number of numeric aliases the server allows.
    pub topic_alias_max: Option<u16>,

    /// Additional human-readable information from the server.
    pub reason_string: Option<String>,

    /// Generic key-value properties.
    pub user_properties: HashMap<String, String>,

    /// 
    pub wildcard_subscription_available: bool,

    ///
    pub subscription_ids_available: bool,

    ///
    pub shared_subscription_available: bool,

    ///
    pub server_keep_alive: Option<u16>,

    /// Application-level instructions on how to build the response topic such as the base of the topic tree.
    pub response_info: Option<String>,

    /// Used in conjunction with [reason code 0x9C](crate::types::ReasonCode::UseAnotherServer) to define a different
    /// server for the client to use.
    pub server_reference: Option<String>,

    /// Application-specific auth method
    pub auth_method: Option<String>,

    /// Application-specific auth data
    pub auth_data: Option<Vec<u8>>,
}

impl TryFrom<&[u8]> for ConnackPacket {
    
    type Error = MqttError;

    fn try_from(src: &[u8]) -> Result<Self, Self::Error> {
        if src[0] != FIRST_BYTE {
            return Err(MqttError::MalformedPacket(format!("First byte not a CONNACK packet: {:08b}", src[0])))
        }

        let remaining_length = VariableByteInteger::try_from(&src[1..5])?;

        // the index where the Variable Header begins
        let mut index = remaining_length.encoded_len() + 1;
        
        // TODO should we actually do something with the session present flag if it is set? check the spec
        let session_present = src[index] != 0;
        index += 1;

        let reason_code = ReasonCode::try_from(src[index])?;
        index += 1;

        let props_length = VariableByteInteger::try_from(&src[index..])?;
        index += props_length.encoded_len();

        let properties = match props_length.value {
            0 => {
                println!("properties length is 0, skipping");
                None
            },
            _=> {
                let index_stop = index + props_length.value as usize;
                println!("parsing {} bytes of properties now", props_length.value);
                Some(ConnackProperties::try_from(&src[index..index_stop])?)
            }
        };
        
        Ok(ConnackPacket { session_present, reason_code, properties })
    }
}

impl Into<Vec<u8>> for ConnackPacket {
    
    fn into(self) -> Vec<u8> {
        let mut packet: Vec<u8> = Vec::new();

        packet.push(FIRST_BYTE);
        packet.push(self.session_present.into());
        packet.push(self.reason_code.into());
        
        match self.properties {
            Some(props) => {
                let mut vec: Vec<u8> = props.into();
                packet.append(&mut vec);
            },
            None => {
                packet.push(0)
            },
        };

        calculate_and_insert_length(&mut packet);

        packet
    }
}

impl MqttControlPacket<'_> for ConnackPacket {
    
    fn packet_type() -> PacketType {
        PacketType::CONNACK
    }
}

impl Default for ConnackProperties {
    fn default() -> Self {
        Self { 
            session_expiry_interval: None, 
            receive_maximum: None, 
            maximum_qos: None, 
            retain_available: true,
            max_packet_size: None,
            assigned_client_id: None,
            topic_alias_max: None,
            reason_string: None,
            user_properties: HashMap::new(),
            wildcard_subscription_available: true,
            subscription_ids_available: true,
            shared_subscription_available: true,
            server_keep_alive: None,
            response_info: None,
            server_reference: None,
            auth_method: None,
            auth_data: None,
        }
    }
}

impl TryFrom<&[u8]> for ConnackProperties {
    type Error = MqttError;

    fn try_from(src: &[u8]) -> Result<Self, Self::Error> {
        let mut properties = ConnackProperties::default();

        let mut cursor = 0;
        while cursor < src.len() {
            let id = src[cursor];
            cursor += 1;

            match id {
                17 => {
                    match src[cursor..cursor + 4].try_into() {
                        Ok(a) => properties.session_expiry_interval = Some(u32::from_be_bytes(a)),
                        Err(e) => return Err(MqttError::Message(format!("Error reading property [session expiry interval]: {:?}", e))),
                    };
                    cursor += 4; // u32
                },
                18 => {
                    let assigned_client_id = UTF8String::try_from(&src[cursor..])?;
                    cursor += assigned_client_id.encoded_len();
                    properties.assigned_client_id = assigned_client_id.value;
                },
                19 => {
                    match src[cursor..cursor + 2].try_into() {
                        Ok(a) => properties.server_keep_alive = Some(u16::from_be_bytes(a)),
                        Err(e) => return Err(MqttError::Message(format!("Error reading property [server keep alive]: {:?}", e))),
                    };
                    cursor += 2; // u16
                },
                21 => {
                    let auth_method = UTF8String::try_from(&src[..])?;
                    cursor += auth_method.encoded_len();
                    properties.auth_method = auth_method.value;
                },
                22 => {
                    let auth_data = BinaryData::try_from(&src[..])?;
                    cursor += auth_data.encoded_len();
                    properties.auth_data = Some(auth_data.clone_inner());
                },
                26 => {
                    let response_info = UTF8String::try_from(&src[cursor..])?;
                    cursor += response_info.encoded_len();
                    properties.response_info = response_info.value;
                },
                28 => {
                    let server_reference = UTF8String::try_from(&src[cursor..])?;
                    cursor += server_reference.encoded_len();
                    properties.server_reference = server_reference.value;
                },
                31 => {
                    let reason_string = UTF8String::try_from(&src[cursor..])?;
                    cursor += reason_string.encoded_len();
                    properties.reason_string = reason_string.value;
                },
                33 => {
                    match src[cursor..cursor + 2].try_into() {
                        Ok(a) => properties.receive_maximum = Some(u16::from_be_bytes(a)),
                        Err(e) => return Err(MqttError::Message(format!("Error reading property [receive max]: {:?}", e))),
                    };
                    
                    cursor += 2; // u16
                },
                34 => {
                    match src[cursor..cursor + 2].try_into() {
                        Ok(a) => properties.topic_alias_max = Some(u16::from_be_bytes(a)),
                        Err(e) => return Err(MqttError::Message(format!("Error reading property [topic alias max]: {:?}", e))),
                    };
                    cursor += 2; // u16
                },
                36 => {
                    properties.maximum_qos = Some(QoS::try_from(src[cursor])?);
                    cursor += 1;
                },
                37 => {
                    let val = src[cursor];
                    match val {
                        0 => properties.retain_available = false,
                        1 => properties.retain_available = true,
                        _=> return Err(MqttError::ProtocolError(format!("illegal value for [retain available]: {}", val))),
                    }
                    cursor += 1; // bool / single byte
                },
                38 => {
                    let key = UTF8String::try_from(&src[cursor..])?;
                    cursor += key.encoded_len();
                    let val = UTF8String::try_from(&src[cursor..])?;
                    cursor += val.encoded_len();

                    // only save this property if we have a key with actual data
                    if let Some(s) = key.value {
                        properties.user_properties.insert(s, val.value.unwrap_or(String::new()));
                    } else {
                        println!("[CONNACK] User Property with empty key found, skipping");
                    }
                },
                39 => {
                    match src[cursor..cursor + 4].try_into() {
                        Ok(a) => properties.max_packet_size = Some(u32::from_be_bytes(a)),
                        Err(e) => return Err(MqttError::Message(format!("Error reading property [max packet size]: {:?}", e))),
                    };
                    cursor += 4; // u32
                },
                40 => {
                    let val = src[cursor];
                    match val {
                        0 => properties.wildcard_subscription_available = false,
                        1 => properties.wildcard_subscription_available = true,
                        _=> return Err(MqttError::ProtocolError(format!("illegal value for [wildcard subscription available]: {}", val))),
                    }
                    cursor += 1; // bool / single byte
                },
                41 => {
                    let val = src[cursor];
                    match val {
                        0 => properties.subscription_ids_available = false,
                        1 => properties.subscription_ids_available = true,
                        _=> return Err(MqttError::ProtocolError(format!("illegal value for [subscription identifiers available]: {}", val))),
                    }
                    cursor += 1; // bool / single byte
                },
                42 => {
                    let val = src[cursor];
                    match val {
                        0 => properties.shared_subscription_available = false,
                        1 => properties.shared_subscription_available = true,
                        _=> return Err(MqttError::ProtocolError(format!("illegal value for [shared subscription available]: {}", val))),
                    }
                    cursor += 1; // bool / single byte
                },
                _=> return Err(MqttError::Message(format!("Unknown CONNACK property identifier: {}", src[0])))
            }
        }
    
        Ok(properties)
    }
}

impl Into<Vec<u8>> for ConnackProperties {
    fn into(self) -> Vec<u8> {
        let mut vec = Vec::new();

        if let Some(s) = self.assigned_client_id {
            encode_and_append(18, Some(UTF8String::from(s)), &mut vec);
        }
        
        encode_and_append(22, self.auth_data, &mut vec);
        encode_and_append(21, self.auth_method, &mut vec);
        if let Some(val) = self.max_packet_size {
            vec.push(39);
            push_be_u32(val, &mut vec);
        }
        if let Some(val) = self.maximum_qos {
            vec.push(36);
            vec.push(val.into());
        }
        if let Some(s) = self.reason_string {
            encode_and_append(31, Some(UTF8String::from(s)), &mut vec);
        }
        if let Some(val) = self.receive_maximum {
            vec.push(33);
            push_be_u16(val, &mut vec);
        }
        if let Some(s) = self.response_info {
            encode_and_append(26, Some(UTF8String::from(s)), &mut vec);
        }
        if let Some(val) = self.server_keep_alive {
            vec.push(19);
            push_be_u16(val, &mut vec);
        }
        if let Some(s) = self.server_reference {
            encode_and_append(28, Some(UTF8String::from(s)), &mut vec);
        }
        if let Some(val) = self.session_expiry_interval {
            vec.push(17);
            push_be_u32(val, &mut vec);
        }
        if !self.shared_subscription_available {
            vec.push(42);
            vec.push(0);
        }
        if !self.subscription_ids_available {
            vec.push(41);
            vec.push(0);
        }
        if let Some(val) = self.topic_alias_max {
            vec.push(34);
            push_be_u16(val, &mut vec);
        }
        if !self.wildcard_subscription_available {
            vec.push(40);
            vec.push(0);
        }
        
        for (k, v) in self.user_properties {
            encode_and_append(38, Some(UTF8StringPair::new(k, v)), &mut vec);
        }

        // insert the length
        let mut result: Vec<u8> = VariableByteInteger { value: vec.len() as u32}.into();
        result.append(&mut vec);

        result
    }
}

fn encode_and_append<T: Into<Vec<u8>>>(identifier: u8, element: Option<T>, target: &mut Vec<u8>) {
    if let Some(val) = element {
        target.push(identifier);
        target.append(&mut val.into())
    }
}

#[cfg(test)]
mod tests {

    use std::vec;

    use super::*;

    #[test]
    fn decode() -> Result<(), MqttError>{
        // the simplest of successful CONNACKs
        run_decode(
            &vec![32, 3, 0, 0, 0], 
            false, 
            ReasonCode::Success,
            false)?;

        // session present flag set
        run_decode(
            &vec![32, 3, 1, 0, 0], 
            true, 
            ReasonCode::Success,
            false)?;

        // Reason code: error
        run_decode(
            &vec![32, 3, 0, 0x80, 0], 
            false, 
            ReasonCode::UnspecifiedError,
            false)?;

        // Reason Code: Bad Auth
        run_decode(
            &vec![32, 3, 1, 0x8C, 0], // bad authentication
            true, 
            ReasonCode::BadAuthenticationMethod,
            false)?;

        // with properties
        let connack = run_decode(
            &vec![32, 14, 0, 0, 11, 19, 255, 45, 34, 0, 10, 33, 0, 20, 42, 0], 
            false, 
            ReasonCode::Success,
            true)?;
        
        let props = connack.properties.unwrap();
        assert_eq!(Some(65325_u16), props.server_keep_alive);
        assert_eq!(Some(10_u16), props.topic_alias_max);
        assert_eq!(Some(20_u16), props.receive_maximum);
        assert_eq!(false, props.shared_subscription_available);

        Ok(())
    }

    #[test]
    fn encode() {
        let connack = ConnackPacket { session_present: false, reason_code: ReasonCode::Success, properties: None };
        let bin: Vec<u8> = connack.into();
        let expected = vec![32, 3, 0, 0, 0];
        assert_eq!(expected, bin);
    }

    #[test]
    fn encode_with_properties() {
        let mut properties = ConnackProperties::default();
        properties.assigned_client_id = Some("generated-123456".into());
        properties.server_keep_alive = Some(135);
        let connack = ConnackPacket { session_present: true, reason_code: ReasonCode::Success, properties: Some(properties) };
        let actual: Vec<u8> = connack.into();
        let expect: Vec<u8> = vec![32, 25, 1, 0, 22, 18, 0, 16, 103, 101, 110, 101, 114, 97, 116, 101, 100, 45, 49, 50, 51, 52, 53, 54, 19, 0, 135];
        assert_eq!(expect, actual);
    }

    fn run_decode(binary: &[u8], session_present: bool, reason_code: ReasonCode, expect_properties: bool) -> Result<ConnackPacket, MqttError> {
        let connack = ConnackPacket::try_from(binary)?;

        assert_eq!(session_present, connack.session_present);
        assert_eq!(reason_code, connack.reason_code);

        match expect_properties {
            true => assert!(connack.properties.is_some()),
            false => assert!(connack.properties.is_none()),
        };

        Ok(connack)
    }
}