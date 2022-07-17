use std::{collections::HashMap};

use crate::{error::MqttError, types::{QoS, BinaryData, UTF8String, VariableByteInteger, MqttDataType, push_be_u16, push_be_u32}};

use super::{MqttControlPacket, PacketType};

/// 23 characters. The spec says longer client IDs _may_ be used, depending on the server, but servers are not
/// required to, so we'll just cap it there for now.
pub const CLIENT_ID_MAX_LENGTH: usize = 23;

/// The static first byte of a CONNECT packet.
const FIRST_BYTE: u8 = 0b00010000;

/// The first 6 bytes of the variable header are, ironically, static.
const PROTO_NAME: [u8; 6] = [0, 4, 77, 81, 84, 84];

/// For now we're only supporting MQTT5
/// TODO add 3.1.1 and 3.1
const PROTO_LEVEL: u8 = 5;

/// CONNECT
/// 
/// Fixed Header (packet type (1) | reserved (0)):
/// 0001 0000
/// [remaining length: VBI] (len(variable_header) + len(payload))
/// 
/// Variable Header:
///     Protocol Name ('MQTT') 
///     Protocol Level (5)
///     Connect Flags (username, pwd, will retain, will qos (2 bits), will flag, clean start, reserved)
///     Keep Alive (2 byte, KA interval in seconds)
///     Properties:
///         length: VBI
///         session expiry interval
///         receive maximum
///         max packet size
///         topic alias max
///         request response info
///         request problem info
///         user property*
///         auth method
///         auth data
/// 
/// Payload:
/// ClientID, Will Props, Will Topic, Will Payload, username, password
#[derive(Debug, PartialEq)]
pub struct ConnectPacket {

    // FIXME this should be Option<> as MQTT5 allows empty client ids (in which case the server will have to generate one)
    client_id: UTF8String,
    protocol_level: u8,
    keep_alive: u16,
    properties: Option<ConnectProperties>,
    will: Option<LastWill>,
    clean_start: bool,
    username: Option<UTF8String>,
    password: Option<BinaryData>,
}

///
#[derive(Debug, PartialEq)]
pub struct ConnectProperties {
    session_expiry_interval: Option<u32>,
    receive_maximum: Option<u16>,
    max_packet_size: Option<u32>,
    topic_alias_max: Option<u16>,
    request_response_info: bool,
    request_problem_info: bool,
    user_properties: HashMap<UTF8String, UTF8String>,
    auth_method: Option<UTF8String>,
    auth_data: Option<Vec<u8>>,
}

/// Last Will and Testament.
#[derive(Debug, PartialEq)]
pub struct LastWill {
    qos: QoS,
    retain: bool,
    will_delay: Option<u32>,
    payload_format_utf8: bool,
    message_expiry_interval: Option<u32>,
    content_type: Option<UTF8String>,
    response_topic: Option<UTF8String>,
    correlation_data: Option<BinaryData>,
    user_properties: HashMap<UTF8String, UTF8String>,
    will_topic: UTF8String,
    will_payload: BinaryData,
}

/// This is used internally during encoding and decoding only.
#[derive(Debug, PartialEq)]
struct ConnectFlags {
    /// If a CONNECT packet is received with Clean Start is set to 1, the Client and Server MUST discard any existing 
    /// Session and start a new Session CONNACK is always set to 0 if Clean Start is set to 1.
    /// 
    /// If a CONNECT packet is received with Clean Start set to 0 and there is a Session associated with the Client 
    /// Identifier, the Server MUST resume communications with the Client based on state from the existing Session 
    /// [MQTT-3.1.2-5]. If a CONNECT packet is received with Clean Start set to 0 and there is no Session associated 
    /// with the Client Identifier, the Server MUST create a new Session [MQTT-3.1.2-6].
    clean_start: bool,

    ///  If the Will Flag is set to 1, the Will Properties, Will Topic, and Will Payload fields MUST be present in the 
    /// Payload [MQTT-3.1.2-9]. The Will Message MUST be removed from the stored Session State in the Server once it 
    /// has been published or the Server has received a DISCONNECT packet with a Reason Code of 0x00 (Normal 
    /// disconnection) from the Client.
    will_flag: bool,

    ///  If the Will Flag is set to 0, then the Will QoS MUST be set to 0 (0x00) [MQTT-3.1.2-11]. 
    /// If the Will Flag is set to 1, the value of Will QoS can be 0 (0x00), 1 (0x01) or 2 (0x02) [MQTT-3.1.2-12]. 
    /// A value of 3 (0x03) is a Malformed Packet.
    will_qos: Option<QoS>,

    /// If the Will Flag is set to 0, then Will Retain MUST be set to 0 [MQTT-3.1.2-13]. If the Will Flag is set to 1 
    /// and Will Retain is set to 0, the Server MUST publish the Will Message as a non-retained message 
    /// [MQTT-3.1.2-14]. If the Will Flag is set to 1 and Will Retain is set to 1, the Server MUST publish the Will 
    /// Message as a retained message [MQTT-3.1.2-15].
    will_retain: bool,

    /// If the Password Flag is set to 0, a Password MUST NOT be present in the Payload [MQTT-3.1.2-18]. 
    /// If the Password Flag is set to 1, a Password MUST be present in the Payload [MQTT-3.1.2-19].
    password_flag: bool,

    /// If the User Name Flag is set to 0, a User Name MUST NOT be present in the Payload [MQTT-3.1.2-16]. 
    /// If the User Name Flag is set to 1, a User Name MUST be present in the Payload [MQTT-3.1.2-17].
    username_flag: bool,
}

impl ConnectPacket {

    // TODO add new() for auto-generated client id

    /// Creates a new `CONNECT` packet with the given `client_id` and defaults values for everything else.
    /// "Defaults" meaning protocol level 5, keep alive of 0, no properties and no will.
    pub fn new(client_id: String) -> Result<Self, MqttError> {
        validate_client_id(&client_id)?;

        Ok(ConnectPacket { 
            client_id: UTF8String::new(client_id), 
            protocol_level: PROTO_LEVEL, 
            keep_alive: 0,
            properties: None,
            will: None,
            clean_start: false,
            username: None,
            password: None,
        })
    }

    /// Inserts or updates a `user property`.
    pub fn set_user_property(&mut self, key: String, value: String) {
        let props = self.properties.get_or_insert(ConnectProperties::new());
        props.user_properties.insert(UTF8String::new(key), UTF8String::new(value));
    }
}

impl MqttControlPacket for ConnectPacket {
    
    fn packet_type() -> PacketType {
        PacketType::CONNECT
    }
}

impl Into<Vec<u8>> for ConnectPacket {

    fn into(self) -> Vec<u8> {
        let mut packet: Vec<u8> = Vec::new();

        // fixed header
        packet.push(FIRST_BYTE);
        // we'll insert the remaining length all the way at the end

        // variable header
        //   - protocol name
        packet.extend_from_slice(&PROTO_NAME);
        //packet.append(&mut UTF8String::from_str("MQTT").into());
        
        //   - protocol version
        packet.push(self.protocol_level);

        // connect flags
        packet.push(ConnectFlags::build(&self).into());

        // keep alive
        for b in self.keep_alive.to_be_bytes() {
            packet.push(b);
        }
        
        // properties
        if self.properties.is_some() {
            packet.append(&mut self.properties.unwrap().into());
        } else {
            packet.push(0);
        }

        // client id
        packet.append(&mut self.client_id.into());

        if let Some(will) = self.will {
            // FIXME include will properties, for now we're just setting them to '0' length
            packet.push(0);
            packet.append(&mut will.will_topic.into());
            packet.append(&mut will.will_payload.into());
        }

        if let Some(uname) = self.username {
            packet.append(&mut uname.into());
        }

        if let Some(pwd) = self.password {
            packet.append(&mut pwd.into());
        }

        super::calculate_and_insert_length(&mut packet);

        packet        
    }
}

impl TryFrom<&[u8]> for ConnectPacket {
    type Error = MqttError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let mut packet = ConnectPacket::new(String::new())?;
        let mut cursor: usize = 0;

        let first_byte = value[cursor];
        if FIRST_BYTE != first_byte {
            return Err(MqttError::MalformedPacket(format!("First byte not a CONNECT packet: {:08b}", first_byte)))
        }

        cursor += 1;
        let mut cursor_stop = cursor + 4;

        let remaining_length = VariableByteInteger::try_from(&value[cursor..cursor_stop])?;
        cursor = cursor + remaining_length.encoded_len();

        let actual_length = (value.len() - cursor) as u32;

        if remaining_length.value > actual_length {
            return Err(MqttError::MalformedPacket(
                format!("Message too short, expected {}, but was {} bytes", remaining_length.value, actual_length)))
        }

        // protocol name and level
        cursor_stop = cursor + 6;
        let proto_name = &value[cursor..cursor_stop];
        cursor = cursor_stop;
        let proto_version: u8 = value[cursor];
        validate_protocol(proto_name, proto_version)?;
        packet.protocol_level = proto_version;

        // Connect flags
        cursor += 1;
        let flags = ConnectFlags::try_from(&value[cursor])?; // TODO actually do something with these flags
        packet.clean_start = flags.clean_start;
        
        cursor += 1;
        cursor_stop = cursor + 2;

        //let mut keep_alive: u16 = 0;
        let keep_alive: u16 = match value[cursor..cursor_stop].try_into() {
            Ok(a) => u16::from_be_bytes(a),
            Err(e) => return Err(MqttError::Message(format!("Error reading [keep alive]: {:?}", e))),
        };
        packet.keep_alive = keep_alive;

        // Properties
        cursor = cursor_stop;
        cursor_stop = cursor + 4;
        
        let props_length = VariableByteInteger::try_from(&value[cursor..cursor_stop])?;

        cursor += props_length.encoded_len();

        match props_length.value {
            0 => println!("Properties length is 0, skipping"),
            _=> {
                cursor_stop = cursor + props_length.value as usize;
                packet.properties = Some(ConnectProperties::try_from(&value[cursor..cursor_stop])?);
                cursor = cursor_stop;
            }
        };

        // PAYLOAD
        // The Payload of the CONNECT packet contains one or more length-prefixed fields, whose presence is determined 
        // by the flags in the Variable Header. These fields, if present, MUST appear in the order 
        //   Client Identifier, 
        //   Will Properties, 
        //   Will Topic, 
        //   Will Payload, 
        //   User Name, 
        //   Password 
        // [MQTT-3.1.3-1].

        // clientID
        let client_id = UTF8String::try_from(&value[cursor..])?; 
        cursor += client_id.encoded_len();
        packet.client_id = client_id;
        
        if flags.will_flag {
            // Will Properties
            let will_props_len = VariableByteInteger::try_from(&value[cursor..])?;
            cursor += will_props_len.encoded_len();

            // we are skipping properties for now
            // FIXME implement will property parsing
            cursor += will_props_len.value as usize;

            // Will Topic
            let will_topic = UTF8String::try_from(&value[cursor..])?;
            cursor += will_topic.encoded_len();

            // Will Payload
            let will_payload = BinaryData::try_from(&value[cursor..])?;
            cursor += will_payload.encoded_len();

            let will = LastWill { 
                qos: flags.will_qos.unwrap_or(QoS::AtLeastOnce),
                retain: flags.will_retain,
                will_delay: None, 
                payload_format_utf8: false, 
                message_expiry_interval: None, 
                content_type: None, 
                response_topic: None, 
                correlation_data: None, 
                user_properties: HashMap::new(), 
                will_topic, 
                will_payload };
            packet.will = Some(will);
        }

        if flags.username_flag {
            let username = UTF8String::try_from(&value[cursor..])?;
            cursor += username.encoded_len();
            packet.username = Some(username);
        }

        if flags.password_flag {
            let pwd = BinaryData::try_from(&value[cursor..])?;
            cursor += pwd.encoded_len();
            packet.password = Some(pwd);
        }

        // the cursor should be at the end of the slice now
        let remaining = value.len() - cursor;
        if remaining > 0 {
            println!("Done parsing CONNECT packet, there are {} bytes left in the input slice", remaining);
        }

        Ok(packet)
    }
}

impl ConnectProperties {
    pub fn new() -> Self {
        ConnectProperties { 
            session_expiry_interval: None, 
            receive_maximum: None, 
            max_packet_size: None, 
            topic_alias_max: None, 
            request_response_info: false, 
            request_problem_info: false, 
            user_properties: HashMap::new(), 
            auth_method: None,
            auth_data: None, 
        }
    }
}

impl TryFrom<&[u8]> for ConnectProperties {
    type Error = MqttError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let mut result = ConnectProperties::new();
        
        let mut cursor: usize = 0;
        while cursor < value.len() {
            cursor += parse_property(&value[cursor..], &mut result)?;
        }

        Ok(result)
    }
}

impl Into<Vec<u8>> for ConnectProperties {
    fn into(self) -> Vec<u8> {
        let mut result = Vec::new();
        
        if let Some(val) = self.session_expiry_interval {
            result.push(17);
            push_be_u32(val, &mut result);
        }

        if let Some(val) = self.receive_maximum {
            result.push(33);
            push_be_u16(val, &mut result);
        }

        if let Some(val) = self.max_packet_size {
            result.push(39);
            push_be_u32(val, &mut result);
        }

        if let Some(val) = self.topic_alias_max {
            result.push(34);
            push_be_u16(val, &mut result)
        }

        if self.request_response_info {
            result.push(25);
            result.push(1);
        }

        if self.request_problem_info {
            result.push(23);
            result.push(1);
        }

        for (k, v) in self.user_properties {
            result.push(38);
            result.append(&mut k.into());
            result.append(&mut v.into());
        }

        if let Some(val) = self.auth_method {
            result.push(21);
            result.append(&mut val.into());
        }

        if let Some(mut val) = self.auth_data {
            result.push(22);
            result.append(&mut val);
        }

        // and finally insert the length at the front
        let length: Vec<u8> = VariableByteInteger{value: result.len() as u32}.into();
        let mut index = 0;
        for b in length {
            result.insert(index, b);
            index += 1;
        }

        result
    }
}

impl LastWill {

    pub fn new(topic: String, payload: &[u8]) -> Result<Self, MqttError> {
        Ok(LastWill { 
            qos: QoS::AtLeastOnce, 
            retain: false,
            will_delay: None,
            payload_format_utf8: false, 
            message_expiry_interval: None, 
            content_type: None, 
            response_topic: None, 
            correlation_data: None, 
            user_properties: HashMap::new(), 
            will_topic: UTF8String::new(topic), 
            will_payload: BinaryData::new(payload.to_vec())? })
    }
}

impl ConnectFlags {
    const CLEAN_START_MASK: u8 = 0b00000010;
    const WILL_FLAG_MASK: u8 = 0b00000100;
    const WILL_RETAIN_MASK: u8 = 0b00100000;
    const PASSWORD_MASK: u8 = 0b01000000;
    const USERNAME_MASK: u8 = 0b10000000;
    const WILL_QOS_MASK: u8 = 0b00000011;
    const WILL_QOS_SHIFT: u8 = 3;

    fn build(packet: &ConnectPacket) -> Self {
        let mut flags = ConnectFlags::default();

        flags.clean_start = packet.clean_start;

        if let Some(w) = &packet.will {
            flags.will_flag = true;
            flags.will_qos = Some(w.qos);
            flags.will_retain = w.retain;
        }

        flags.username_flag = packet.username.is_some();
        flags.password_flag = flags.username_flag && packet.password.is_some();

        flags
    }
}

impl Default for ConnectFlags {
    fn default() -> Self {
        Self { 
            clean_start: true, 
            will_flag: false, 
            will_qos: None, 
            will_retain: false, 
            password_flag: false, 
            username_flag: false 
        }
    }
}

impl TryFrom<&u8> for ConnectFlags {
    type Error = MqttError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        // validate reserved is 0
        if (value & 0b0000001) != 0 {
            return Err(MqttError::MalformedPacket("Reserved connect flag must be unset".to_string()))
        }
        
        let clean_start = (value & Self::CLEAN_START_MASK) != 0;
        let will_flag = (value & Self::WILL_FLAG_MASK) != 0;
        let will_retain = (value & Self::WILL_RETAIN_MASK) != 0;
        let password_flag = (value & Self::PASSWORD_MASK) != 0;
        let username_flag = (value & Self::USERNAME_MASK) != 0;
        let will_qos_raw: u8 = (value >> Self::WILL_QOS_SHIFT) & Self::WILL_QOS_MASK;

        let will_qos = match will_flag {
            true => Some(QoS::try_from(will_qos_raw)?),
            false => {
                if will_qos_raw != 0 {
                    return Err(MqttError::MalformedPacket("Will QoS may only be set if will flag is set (MQTT-3.1.2-11)".to_string()))
                }
                None
            },
        };

        Ok(ConnectFlags { clean_start, will_flag, will_qos, will_retain, password_flag, username_flag})
    }
}

impl Into<u8> for ConnectFlags {
    fn into(self) -> u8 {
        let mut result = 0b00000000;
        if self.clean_start {
            result |= Self::CLEAN_START_MASK;
        }
        if self.will_flag {
            result |= Self::WILL_FLAG_MASK;
        }
        if self.will_retain {
            result |= Self::WILL_RETAIN_MASK;
        }
        if self.username_flag {
            result |= Self::USERNAME_MASK;
        }
        if self.password_flag {
            result |= Self::PASSWORD_MASK;
        }

        if self.will_qos.is_some() {
            let qos: u8 = self.will_qos.unwrap().into();
            result |= qos << Self::WILL_QOS_SHIFT;
        }

        result
    }
}

fn validate_protocol(name: &[u8], level: u8) -> Result<(), MqttError> {
    for (expect, actual) in name.iter().zip(PROTO_NAME.iter()) {
        if expect != actual {
            // TODO maybe convert to UTF8String for display reasons?
            return Err(MqttError::MalformedPacket(format!("Invalid Protocol Name sequence: {:?}", name)))
        }
    }

    if level != PROTO_LEVEL {
        return Err(MqttError::MalformedPacket(format!("Unsupported protocol level: {}", level)))
    }

    Ok(())
}

fn validate_client_id(client_id: &String) -> Result<(), MqttError> {
    if !client_id.is_ascii() {
        return Err(MqttError::Message("ClientID may only contain alphanumeric ASCII characters".to_string()))
    } else if client_id.len() > CLIENT_ID_MAX_LENGTH {
        return Err(MqttError::Message(format!("ClientID should not exceed {} characters", CLIENT_ID_MAX_LENGTH)))
    }

    Ok(())
}

/// TODO clean this up, this is ugly AF. IDs and data types of properties are finite, so we should be able to solve this a little more generic
fn parse_property(src: &[u8], properties: &mut ConnectProperties) -> Result<usize, MqttError> {
    let mut bytes_used = 1; // we're always reading the ID field;
    let (identifier, remain) = src.split_at(1);
    match identifier[0] {
        17 => {
            match remain[..4].try_into() {
                Ok(a) => properties.session_expiry_interval = Some(u32::from_be_bytes(a)),
                Err(e) => return Err(MqttError::Message(format!("Error reading property [session expiry interval]: {:?}", e))),
            };
            bytes_used += 4; // u32
        },
        21 => {
            let auth_method = UTF8String::try_from(remain)?;
            bytes_used += auth_method.encoded_len();
            properties.auth_method = Some(auth_method);
        },
        22 => {
            let auth_data = BinaryData::try_from(remain)?;
            bytes_used += auth_data.encoded_len();
            properties.auth_data = Some(auth_data.clone_inner());
        },
        23 => {
            match remain[0] {
                0 => properties.request_problem_info = false,
                1 => properties.request_problem_info = true,
                _=> return Err(MqttError::ProtocolError(format!("illegal value for [request problem info]: {}", remain[0]))),
            }
            bytes_used += 1; // bool / single byte
        },
        25 => {
            match remain[0] {
                0 => properties.request_response_info = false,
                1 => properties.request_response_info = true,
                _=> return Err(MqttError::ProtocolError(format!("illegal value for [request response info]: {}", remain[0]))),
            }
            bytes_used += 1; // bool / single byte
        },
        33 => {
            match remain[..2].try_into() {
                Ok(a) => properties.receive_maximum = Some(u16::from_be_bytes(a)),
                Err(e) => return Err(MqttError::Message(format!("Error reading property [receive max]: {:?}", e))),
            };
            
            bytes_used += 2; // u16
        },
        34 => {
            match remain[..2].try_into() {
                Ok(a) => properties.topic_alias_max = Some(u16::from_be_bytes(a)),
                Err(e) => return Err(MqttError::Message(format!("Error reading property [topic alias max]: {:?}", e))),
            };
            bytes_used += 2; // u16
        },
        38 => {
            let key = UTF8String::try_from(remain)?;
            bytes_used += key.encoded_len();
            let val = UTF8String::try_from(&remain[key.encoded_len()..])?;
            bytes_used += val.encoded_len();
            properties.user_properties.insert(key, val);
        },
        39 => {
            match remain[..4].try_into() {
                Ok(a) => properties.max_packet_size = Some(u32::from_be_bytes(a)),
                Err(e) => return Err(MqttError::Message(format!("Error reading property [max packet size]: {:?}", e))),
            };
            bytes_used += 4; // u32
        },
        _=> return Err(MqttError::Message(format!("Unknown CONNECT property identifier: {}", src[0])))
    }
    Ok(bytes_used)
}

#[cfg(test)]
mod tests {

    use std::str::FromStr;

    use super::*;

    #[test]
    fn encode() {
        let client_id = "ENCTST";
        let mut conn = ConnectPacket::new(client_id.to_string()).unwrap();
        conn.clean_start = true;

        let binary: Vec<u8> = conn.into();
        assert!(binary.len() > 0);

        let expect: Vec<u8> = vec![
            FIRST_BYTE,
            19, // remaining length in no of bytes
            PROTO_NAME[0], PROTO_NAME[1], PROTO_NAME[2], PROTO_NAME[3], PROTO_NAME[4], PROTO_NAME[5], 
            PROTO_LEVEL,
            0b00000010, // connect flags
            0, 0, // keep alive
            0, // property length
            0, 6, 69, 78, 67, 84, 83, 84 // client ID
        ];

        assert_eq!(expect, binary);
    }

    #[test]
    fn encode_with_will() {
        let expect: Vec<u8> = vec![16,87,0,4,77,81,84,84,5,238,0,60,8,17,0,0,0,120,33,0,1,0,0,0,0,10,47,108,97,115,116,47,119,105,108,108,0,29,123,39,115,39,58,39,115,101,110,115,111,114,39,44,39,108,39,58,39,107,105,116,99,104,101,110,39,32,125,0,6,109,121,110,97,109,101,0,12,115,117,112,101,114,83,101,99,114,101,116,33];
        let mut packet = ConnectPacket::new(String::new()).unwrap();
        packet.clean_start = true;
        packet.keep_alive = 60;
        packet.username = Some(UTF8String::from_str("myname"));
        packet.password = Some(BinaryData::new(String::from_str("superSecret!").unwrap().as_bytes().to_vec()).unwrap());
        
        let mut properties = ConnectProperties::new();
        properties.session_expiry_interval = Some(120);
        properties.receive_maximum = Some(1);
        packet.properties = Some(properties);

        let mut will = LastWill::new(
            "/last/will".to_string(),
            "{'s':'sensor','l':'kitchen' }".to_string().as_bytes()
        ).unwrap();
        will.retain = true;
        packet.will = Some(will);

        let actual: Vec<u8> = packet.into();
        assert_eq!(expect, actual);
    }

    #[test]
    fn decode() {
        //let binary: Vec<u8> = vec![16,19,0,4,77,81,84,84,5,2,0,0,0,0,6,87,85,80,80,68,73];
        // message generated by mosquitto client
        // flags: clean start
        // keep alive: 60
        // properties: receive max (32) and user prop (origin:sensor)
        // no will
        // clientID: DECODE
        let binary: Vec<u8> = vec![16,39,0,4,77,81,84,84,5,2,0,60,20,33,0,32,38,0,6,111,114,105,103,105,110,0,6,115,101,110,115,111,114,0,6,68,69,67,79,68,69];
        let decoded = ConnectPacket::try_from(&binary[..]).unwrap();
        
        // HEADER/FLAGS
        assert_eq!(60, decoded.keep_alive);
        assert!(decoded.clean_start);

        // PROPERTIES
        assert!(decoded.properties.is_some(), "expected CONNECT properties to be present");
        let props = decoded.properties.as_ref().unwrap();
        assert_eq!(Some(32_u16), props.receive_maximum);
        assert_eq!(1, props.user_properties.len());
        assert_eq!(Some(&UTF8String::from_str("sensor")), props.user_properties.get(&UTF8String::from_str("origin")));
        assert!(props.auth_method.is_none());
        assert!(props.auth_data.is_none());
        assert!(props.max_packet_size.is_none());
        assert!(props.session_expiry_interval.is_none());
        assert!(props.topic_alias_max.is_none());
        assert_eq!(false, props.request_problem_info);
        assert_eq!(false, props.request_response_info);

        // PAYLOAD
        assert!(decoded.will.is_none(), "did not exepct a will");
        assert_eq!(UTF8String::from_str("DECODE"), decoded.client_id);

    }

    #[test]
    fn decode_auth() {
        let binary = vec![16,38,0,4,77,81,84,84,5,2,0,60,19,21,0,5,66,65,83,73,67,22,0,8,0,1,2,3,4,5,6,7,0,6,65,85,84,72,73,68];
        let decoded = ConnectPacket::try_from(&binary[..]).unwrap();
        let props = decoded.properties.unwrap();
        
        assert_eq!(Some(UTF8String::from_str("BASIC")), props.auth_method);
        assert_eq!(Some(vec![0,1,2,3,4,5,6,7]), props.auth_data);
    }

    #[test]
    fn decode_will() {
        let binary = vec![16,87,0,4,77,81,84,84,5,238,0,60,8,17,0,0,0,120,33,0,1,0,0,0,0,10,47,108,97,115,116,47,119,105,108,108,0,29,123,39,115,39,58,39,115,101,110,115,111,114,39,44,39,108,39,58,39,107,105,116,99,104,101,110,39,32,125,0,6,109,121,110,97,109,101,0,12,115,117,112,101,114,83,101,99,114,101,116,33];
        let decoded = ConnectPacket::try_from(&binary[..]).unwrap();
        assert_eq!(Some(UTF8String::from_str("myname")), decoded.username);
        assert!(decoded.clean_start);
        assert_eq!(60_u16, decoded.keep_alive);

        let properties = decoded.properties.expect("Properties should have been decoded");
        assert_eq!(Some(120_u32), properties.session_expiry_interval);

        let will = decoded.will.expect("Last Will should have been decoded!");
        assert_eq!(UTF8String::from_str("/last/will"), will.will_topic);
        
        let will_payload = String::from_utf8(will.will_payload.clone_inner()).unwrap();
        assert_eq!("{'s':'sensor','l':'kitchen' }".to_string(), will_payload);

        assert_eq!(QoS::AtLeastOnce, will.qos);
    }

    #[test]
    fn decoding_errors() {
        // first byte does not match the spec
        decode_expect_error(
            vec![17], 
            MqttError::MalformedPacket(format!("First byte not a CONNECT packet: 00010001")));
        
        // message is shorter than the 'remeinaing length' field signifies
        decode_expect_error(
            vec![16,19,0,4,77,81,84,84,5,2,0,0,0,0,6,87,85,80,80,68],
            MqttError::MalformedPacket(format!("Message too short, expected 19, but was 18 bytes")));

        // invalid protocol name
        decode_expect_error(
            vec![16,19,0,4,77,81,84,83,5,2,0,0,0,0,6,87,85,80,80,68,73],
            MqttError::MalformedPacket(format!("Invalid Protocol Name sequence: [0, 4, 77, 81, 84, 83]")));

        // unsupported proto level
        decode_expect_error(
            vec![16,19,0,4,77,81,84,84,4,2,0,0,0,0,6,87,85,80,80,68,73],
            MqttError::MalformedPacket(format!("Unsupported protocol level: 4")));
    }

    fn decode_expect_error(binary: Vec<u8>, expect: MqttError) {
        let result = ConnectPacket::try_from(&binary[..]);
        assert!(result.is_err());
        assert_eq!(Some(expect), result.err());
    }

    #[test]
    fn client_id_validation() {
        assert!(ConnectPacket::new("abcäÖŁ".to_string()).is_err());
        assert!(ConnectPacket::new("abncjidLJKLÄSDU134".to_string()).is_err());
        assert!(ConnectPacket::new("ClientIDIsTooLongSaysSpec".to_string()).is_err());
        
        // not really sure about this rule, actually
        //assert!(ConnectPacket::new("no whitespace allowed".to_string()).is_err());
        assert!(ConnectPacket::new("perfectly_valid".to_string()).is_ok());
    }

    #[test]
    fn user_properties() {
        let mut packet = ConnectPacket::new("user_properties_test".to_string()).unwrap();
        packet.set_user_property("onekey".to_string(), "oneval".to_string());
        packet.set_user_property("twokey".to_string(), "twoval".to_string());

        assert_eq!(2, packet.properties.unwrap().user_properties.len());
        // TODO actually assert the contents of the properties map
    }

    #[test]
    fn decode_connect_flags() {
        let mut map: HashMap<u8, ConnectFlags> = HashMap::new();
        map.insert(0b00000010, ConnectFlags{ clean_start: true, will_flag: false, will_qos: None, will_retain: false, password_flag: false, username_flag: false });
        map.insert(0b11101110, ConnectFlags{ clean_start: true, will_flag: true, will_qos: Some(QoS::AtLeastOnce), will_retain: true, password_flag: true, username_flag: true });
        map.insert(0b00110100, ConnectFlags{ clean_start: false, will_flag: true, will_qos: Some(QoS::ExactlyOnce), will_retain: true, password_flag: false, username_flag: false});
        
        for (k, v) in map {
            let flags = ConnectFlags::try_from(&k);
            assert!(flags.is_ok(), "Expected decoded CONNECT flags, but got error: {:?}", flags.err());
            assert_eq!(v, flags.unwrap());
        }
    }

    #[test]
    fn decode_connect_flags_reserved_validation() {
        let res = ConnectFlags::try_from(&0b00000001);
        assert!(res.is_err());
        // FIXME add some more detailed validation
    }

    #[test]
    fn decode_connect_flags_invalid_qos() {
        let res = ConnectFlags::try_from(&0b00001000);
        assert!(res.is_err());
        // FIXME add some more detailed validation
    }

    #[test]
    fn encode_connect_flags() {
        let flags_unset = ConnectFlags{ username_flag: false, password_flag: false, will_flag: false, will_qos: None, will_retain: false, clean_start: false };
        assert_eq!(0b00000000_u8, flags_unset.into());

        let flags_set = ConnectFlags{ username_flag: true, password_flag: true, will_flag: true, will_qos: Some(QoS::ExactlyOnce), will_retain: true, clean_start: true };
        assert_eq!(0b11110110_u8, flags_set.into());
        
        let flags_will_only = ConnectFlags{ username_flag: false, password_flag: false, will_flag: true, will_qos: Some(QoS::AtLeastOnce), will_retain: false, clean_start: false };
        assert_eq!(0b00001100_u8, flags_will_only.into());
    }
}