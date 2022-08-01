//! Types representing the `CONNECT` control packet, which must be the first packet sent by a client when making *or*
//! re-establishing a server connection.

use std::collections::HashMap;

use mqtt_derive::MqttProperties;

use crate::{error::MqttError, types::{QoS, BinaryData, UTF8String, MqttDataType}};

use super::{MqttControlPacket, PacketType, Decodeable, DecodingResult, remaining_length};

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

/// A `CONNECT` MQTT control packet with support for encoding into and decoding from its binary format.
/// 
/// # Examples
/// 
/// A very basic CONNECT, without a client ID (will be generated by the server).
/// ```
/// use mqtt::packet::Connect;
/// 
/// let mut packet = Connect::default();
/// packet.keep_alive = 77;
/// 
/// // add more stuff here...
/// 
/// let encoded: Vec<u8> = packet.into();
/// 
/// let decoded = Connect::try_from(&encoded[..]).unwrap();
/// assert_eq!(77, decoded.keep_alive);
/// ```
/// 
/// To specify a client ID:
/// ```
/// use mqtt::packet::Connect;
/// 
/// let packet = Connect::with_client_id_str("my-client-id").unwrap();
/// ```
/// 
/// # Binary Format
/// 
/// See the [MQTT spec](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901033) for details on
/// the binary format.
#[derive(Debug, PartialEq)]
pub struct Connect {
    
    /// Starting with version 5, MQTT allows sending an empty client ID, in which case one will be appointed by the 
    /// server, which must be used. See [CONNACK](super::Connack) for details.
    pub client_id: Option<String>,

    /// FIXME this really should be bounded as an enum or similar
    protocol_level: u8,

    /// Number of seconds before the server closes the connection unless the client has sent another packet.
    /// This value is a request by the client, the [server can override this in the response](super::Connack).
    pub keep_alive: u16,

    /// Whether to start a new session or resume an existing one (if it exists).
    pub clean_start: bool,

    /// MQTT 5 supports optional properties, [see here for details](ConnectProperties).
    pub properties: Option<ConnectProperties>,

    /// An optional message to be published by the server in case it closes the connection to the client in case of 
    /// inactivity.
    pub will: Option<LastWill>,

    /// Authentication towards the server.
    pub username: Option<String>,

    /// Authentication towards the server.
    pub password: Option<Vec<u8>>,
}

/// Optional property values for the `CONNECT` packet.
#[derive(Debug, PartialEq, MqttProperties)]
pub struct ConnectProperties {
    /// How long a previously established session may be picked up after connection loss in seconds.
    /// Defaults to '0'.
    pub session_expiry_interval: Option<u32>,

    /// Max number of concurrent QoS 1 and 2 publications the client can handle.
    pub receive_maximum: Option<u16>,

    /// Number of bytes per message, representing the maximum a client is willing to accept.
    /// Servers that support this option will discard any messages larger than this value!
    /// Defaults to undefined, meaning no limits on packet size.
    pub maximum_packet_size: Option<u32>,

    /// TODO we don't support topic aliases yet :(
    pub topic_alias_maximum: Option<u16>,

    /// If set to `true`, the server _may_ include additional information in the [CONNACK response](super::Connack).
    /// Defaults to `false`.
    pub request_response_information: Option<bool>,

    /// If set to `true`, the server _may_ include additional information in any subsequent messages. This includes 
    /// the `reason string` and `user properties`.
    /// Defaults to `false`.    
    pub request_problem_information: Option<bool>,

    /// Application-specific key-value elements.
    pub user_property: HashMap<String, String>,

    /// Application-specific auth method definition.
    pub authentication_method: Option<String>,

    /// Application-specific auth data.
    pub authentication_data: Option<Vec<u8>>,
}

/// An MQTT message (including properties) that is published by the broker in case it "loses" connection to the client.
/// The client specifies topic, payload and properties with the connection itself.
#[derive(Debug, PartialEq)]
pub struct LastWill {
    /// Quality of Service for the will message.
    pub qos: QoS,

    /// Whether or not the server should retain the will message after publishing it.
    pub retain: bool,

    pub properties: Option<WillProperties>,

    /// Name of the topic to publish this will message to.
    pub will_topic: String,

    /// The actual will message, in an application-specific format.
    /// Also see [WillProperties::payload_format_indicator] and [WillProperties::content_type].
    pub will_payload: Vec<u8>,
}

#[derive(Debug, PartialEq, MqttProperties)]
pub struct WillProperties {
    
    /// The grace period (in seconds) after the server has determined it has lost connection to the client before it 
    /// publishes the will message.
    /// This allows clients to reconnect after having "missed" the keep alive interval.
    pub will_delay_interval: Option<u32>,

    /// Whether the format of the payload is a UTF-8 compliant string or just a bunch of bytes.
    /// Defaults to `false` (bunch of bytes). Servers _may_ validate that the payload is actually well-formed UTF-8 if
    /// this value is `true`.    
    pub payload_format_indicator: Option<bool>,

    /// The lifetime of the will message in seconds.    
    pub message_expiry_interval: Option<u32>,

    /// Application-specific content type definition of the payload. Note that this has nothing to do with 
    /// [Self::payload_format_indicator].
    pub content_type: Option<String>,

    /// Response topic definiton.
    pub response_topic: Option<String>,

    /// Application-specific data.
    pub correlation_data: Option<Vec<u8>>,

    /// Name of the topic to publish this will message to.
    pub user_property: HashMap<String, String>,
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

impl Connect {

    /// Validates the provided client ID for length and content (ASCII only) and then creates an otherwise default
    /// packet along with that id.
    pub fn with_client_id(client_id: String) -> Result<Self, MqttError> {
        validate_client_id(&client_id)?;
        let mut packet = Connect::default();
        packet.client_id = Some(client_id);
        Ok(packet)
    }

    /// Convenience for `with_client_id(client_id.to_string())`.
    /// Same validation rules as wth [Connect::with_client_id] still apply.
    pub fn with_client_id_str(client_id: &str) -> Result<Self, MqttError> {
        Self::with_client_id(client_id.to_string())
    }

    /// Inserts or updates a `user property`.
    pub fn set_user_property(&mut self, key: String, value: String) {
        let props = self.properties.get_or_insert(ConnectProperties::default());
        props.user_property.insert(key, value);
    }
}

impl MqttControlPacket<'_> for Connect {
    
    fn packet_type() -> PacketType {
        PacketType::CONNECT
    }
}

impl Default for Connect {
    /// Creates an "empty" `CONNECT` packet, without a `client ID`, no will, no properties and a keep alive value of 0.
    fn default() -> Self {
        Self { 
            client_id: None, 
            protocol_level: PROTO_LEVEL, 
            keep_alive: 0,
            properties: None,
            will: None,
            clean_start: false,
            username: None,
            password: None,
        }
    }
}

impl Into<Vec<u8>> for Connect {

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
        if let Some(p) = self.properties {
            packet.append(&mut p.into())
        } else {
            packet.push(0)
        }

        // client id
        let client_id = match self.client_id {
            Some(s) => UTF8String::from(s),
            None => UTF8String::new(),
        };
        packet.append(&mut client_id.into());

        if let Some(will) = self.will {
            // FIXME include will properties, for now we're just setting them to '0' length
            match will.properties {
                Some(props) => packet.append(&mut props.into()),
                None => packet.push(0),
            }

            packet.append(&mut UTF8String::from(will.will_topic).into());
            // FIXME just letting this panic isn't really elegant
            let payload = BinaryData::new(will.will_payload).unwrap();
            packet.append(&mut payload.into());
        }

        if let Some(uname) = self.username {
            packet.append(&mut UTF8String::from(uname).into());
        }

        if let Some(pwd) = self.password {
            let password = BinaryData::new(pwd).unwrap();
            packet.append(&mut password.into());
        }

        super::calculate_and_insert_length(&mut packet);

        packet        
    }
}

impl TryFrom<&[u8]> for Connect {
    type Error = MqttError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let mut packet = Connect::default();
        let mut cursor: usize = 0;

        match value[cursor] {
            FIRST_BYTE => cursor += 1,
            els => return Err(MqttError::MalformedPacket(format!("First byte not a CONNECT packet: {:08b}", els)))
        }

        let remaining_length = remaining_length(&value[cursor..])?;
        cursor += remaining_length.encoded_len();

        // protocol name and level
        let mut cursor_stop = cursor + 6;
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
        let prop_res: DecodingResult<ConnectProperties> = ConnectProperties::decode(&value[cursor..])?;
        cursor += prop_res.bytes_read();
        packet.properties = prop_res.value();

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
        packet.client_id = client_id.value;
        
        if flags.will_flag {
            // Will Properties
            let will_props_res: DecodingResult<WillProperties> = WillProperties::decode(&value[cursor..])?;
            cursor += will_props_res.bytes_read();

            // Will Topic
            let will_topic = UTF8String::try_from(&value[cursor..])?;
            cursor += will_topic.encoded_len();

            // Will Payload
            let will_payload = BinaryData::try_from(&value[cursor..])?;
            cursor += will_payload.encoded_len();

            let will = LastWill { 
                qos: flags.will_qos.unwrap_or(QoS::AtLeastOnce),
                retain: flags.will_retain,
                properties: will_props_res.value(),
                will_topic: will_topic.into(), 
                will_payload: will_payload.clone_inner(),
            };
            packet.will = Some(will);
        }

        if flags.username_flag {
            let username = UTF8String::try_from(&value[cursor..])?;
            cursor += username.encoded_len();
            if let Some(uname) = username.value {
                packet.username = Some(uname)
            }
        }

        if flags.password_flag {
            let pwd = BinaryData::try_from(&value[cursor..])?;
            cursor += pwd.encoded_len();
            packet.password = Some(pwd.clone_inner());
        }

        // the cursor should be at the end of the slice now
        let remaining = value.len() - cursor;
        if remaining > 0 {
            println!("Done parsing CONNECT packet, there are {} bytes left in the input slice", remaining);
        }

        Ok(packet)
    }
}

impl LastWill {

    pub fn new(topic: String, payload: &[u8]) -> Result<Self, MqttError> {
        Ok(LastWill { 
            qos: QoS::AtLeastOnce, 
            retain: false,
            properties: None,
            will_topic: topic, 
            will_payload: payload.to_vec() })
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

    fn build(packet: &Connect) -> Self {
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

impl From<ConnectFlags> for u8 {
    fn from(flags: ConnectFlags) -> Self {
        let mut result = 0b00000000;
        if flags.clean_start {
            result |= ConnectFlags::CLEAN_START_MASK;
        }
        if flags.will_flag {
            result |= ConnectFlags::WILL_FLAG_MASK;
        }
        if flags.will_retain {
            result |= ConnectFlags::WILL_RETAIN_MASK;
        }
        if flags.username_flag {
            result |= ConnectFlags::USERNAME_MASK;
        }
        if flags.password_flag {
            result |= ConnectFlags::PASSWORD_MASK;
        }

        if flags.will_qos.is_some() {
            let qos: u8 = flags.will_qos.unwrap().into();
            result |= qos << ConnectFlags::WILL_QOS_SHIFT;
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

/// TODO: add test(s) for LastWill with properties
#[cfg(test)]
mod tests {

    use std::str::FromStr;

    use super::*;

    #[test]
    fn encode_and_decode() {
        let mut packet = Connect::default();
        packet.keep_alive = 77;
        let encoded: Vec<u8> = packet.into();
        let decoded = Connect::try_from(&encoded[..]).unwrap();
        assert_eq!(77, decoded.keep_alive);
    }

    #[test]
    fn encode() {
        let mut conn = Connect::with_client_id_str("ENCTST").unwrap();
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
        let expect: Vec<u8> = vec![16,86,0,4,77,81,84,84,5,238,0,60,8,17,0,0,0,120,33,0,1,0,0,0,0,10,47,108,97,115,116,47,119,105,108,108,0,28,123,34,115,34,58,34,115,101,110,115,111,114,34,44,34,108,34,58,34,107,105,116,99,104,101,110,34,125,0,6,109,121,110,97,109,101,0,12,115,117,112,101,114,83,101,99,114,101,116,33];
        let mut packet = Connect::default();
        packet.clean_start = true;
        packet.keep_alive = 60;
        packet.username = Some("myname".into());
        packet.password = Some(String::from_str("superSecret!").unwrap().as_bytes().to_vec());
        
        let mut properties = ConnectProperties::default();
        properties.session_expiry_interval = Some(120);
        properties.receive_maximum = Some(1);
        packet.properties = Some(properties);

        let mut will = LastWill::new(
            "/last/will".to_string(),
            r#"{"s":"sensor","l":"kitchen"}"#.to_string().as_bytes()
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
        let decoded = Connect::try_from(&binary[..]).unwrap();
        
        // HEADER/FLAGS
        assert_eq!(60, decoded.keep_alive);
        assert!(decoded.clean_start);

        // PROPERTIES
        assert!(decoded.properties.is_some(), "expected CONNECT properties to be present");
        let props = decoded.properties.as_ref().unwrap();
        assert_eq!(Some(32_u16), props.receive_maximum);
        assert_eq!(1, props.user_property.len());
        assert_eq!(Some(&String::from_str("sensor").unwrap()), props.user_property.get("origin".into()));
        assert!(props.authentication_method.is_none());
        assert!(props.authentication_data.is_none());
        assert!(props.maximum_packet_size.is_none());
        assert!(props.session_expiry_interval.is_none());
        assert!(props.topic_alias_maximum.is_none());
        assert!(props.request_problem_information.is_none());
        assert!(props.request_response_information.is_none());

        // PAYLOAD
        assert!(decoded.will.is_none(), "did not exepct a will");
        assert_eq!(Some("DECODE".into()), decoded.client_id);

    }

    #[test]
    fn decode_auth() {
        let binary = vec![16,38,0,4,77,81,84,84,5,2,0,60,19,21,0,5,66,65,83,73,67,22,0,8,0,1,2,3,4,5,6,7,0,6,65,85,84,72,73,68];
        let decoded = Connect::try_from(&binary[..]).unwrap();
        assert!(decoded.properties.is_some(), "expected properties to be decoded as well!");
        let props = decoded.properties.unwrap();
        
        assert_eq!(Some("BASIC".into()), props.authentication_method);
        assert_eq!(Some(vec![0,1,2,3,4,5,6,7]), props.authentication_data);
    }

    #[test]
    fn decode_will() {
        let binary = vec![16,86,0,4,77,81,84,84,5,238,0,60,8,17,0,0,0,120,33,0,1,0,0,0,0,10,47,108,97,115,116,47,119,105,108,108,0,28,123,34,115,34,58,34,115,101,110,115,111,114,34,44,34,108,34,58,34,107,105,116,99,104,101,110,34,125,0,6,109,121,110,97,109,101,0,12,115,117,112,101,114,83,101,99,114,101,116,33];
        let decoded = Connect::try_from(&binary[..]).unwrap();
        assert_eq!(Some("myname".into()), decoded.username);
        assert!(decoded.clean_start);
        assert_eq!(60_u16, decoded.keep_alive);

        let properties = decoded.properties.expect("Properties should have been decoded");
        assert_eq!(Some(120_u32), properties.session_expiry_interval);

        let will = decoded.will.expect("Last Will should have been decoded!");
        assert_eq!("/last/will".to_string(), will.will_topic);

        let will_payload = String::from_utf8(will.will_payload).unwrap();
        assert_eq!(r#"{"s":"sensor","l":"kitchen"}"#.to_string(), will_payload);

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
        let result = Connect::try_from(&binary[..]);
        assert!(result.is_err());
        assert_eq!(Some(expect), result.err());
    }

    #[test]
    fn client_id_validation() {
        assert!(Connect::with_client_id_str("abcäÖŁ").is_err());
        assert!(Connect::with_client_id_str("abncjidLJKLÄSDU134").is_err());
        assert!(Connect::with_client_id_str("ClientIDIsTooLongSaysSpec").is_err());
        
        // not really sure about this rule, actually
        //assert!(ConnectPacket::new("no whitespace allowed".to_string()).is_err());
        assert!(Connect::with_client_id_str("perfectly_valid").is_ok());
    }

    #[test]
    fn user_properties() {
        let mut packet = Connect::with_client_id_str("user_properties_test").unwrap();
        packet.set_user_property("onekey".to_string(), "oneval".to_string());
        packet.set_user_property("twokey".to_string(), "twoval".to_string());

        assert_eq!(2, packet.properties.unwrap().user_property.len());
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