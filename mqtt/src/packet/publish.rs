use std::collections::HashMap;

use mqtt_derive::MqttProperties;

use crate::{types::{QoS, VariableByteInteger, UTF8String, MqttDataType}, error::MqttError};

use super::{remaining_length, Decodeable, DecodingResult, MqttControlPacket, properties};

/// An MQTT `PUBLISH` packet is used to send a specific message to a topic.
/// 
/// # Examples
/// 
/// ```
/// use mqtt::packet::Publish;
/// 
/// let publish = Publish::new(
///     "/some/topic/name".into(),
///     vec![0, 1, 2, 3, 4],
/// );
/// 
/// ```
///  
#[derive(Debug)]
pub struct Publish {
    // FIXED HEADER
    /// If `true` this message is considered an attempted re-delivery.
    /// Defaults to `false`, and **must** be so if QoS is `0`.
    pub dup: bool,

    /// QoS for this message.
    pub qos_level: QoS,

    /// Whether the server should keep this message for future subscribers or not.
    /// Defaults to `false`.
    /// Also read the spec on a lot more additional info about message retention.
    pub retain: bool,

    // VARIABLE HEADER

    /// Name of the topic to publish to. This is obviously mandatory.
    pub topic_name: String,

    /// Packet identifiers act as sort of a correlation ID for messages within a sequence such as
    /// `PUBLISH` --> `PUBACK`. See the spec, I honestly don't get the point of this, but:
    ///
    /// - a `PUBLISH` packet **MUST NOT** contain a Packet Identifier if its QoS value is set to 0 [MQTT-2.2.1-2].
    /// - a server or client **must** send a new non-zero identifier if publishing a message where QoS > 0.
    /// 
    /// This field is only included in the binary message if QoS is > 0. The value by default is set to `0`.
    pub packet_identifier: Option<u16>,

    /// MQTT5 optional properties.
    pub properties: Option<PublishProperties>,

    // PAYLOAD

    /// The Payload contains the Application Message that is being published. The content and format of the
    /// data is application specific. The length of the Payload can be calculated by subtracting the length of the
    /// Variable Header from the Remaining Length field that is in the Fixed Header. It is valid for a PUBLISH
    /// packet to contain a zero length Payload.
    pub payload: Vec<u8>,
}

/// See [the MQTT spec](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html) about properties.
#[derive(Debug, MqttProperties)]
pub struct PublishProperties {
    pub payload_format_indicator: Option<bool>,
    pub message_expiry_interval: Option<u32>,
    pub topic_alias: Option<u16>,
    pub response_topic: Option<String>,
    pub correlation_data: Option<Vec<u8>>,
    pub user_property: HashMap<String, String>,
    pub subscription_identifier: Option<VariableByteInteger>,
    pub content_type: Option<String>,
}

impl MqttControlPacket<'_> for Publish {
    fn packet_type() -> super::PacketType {
        super::PacketType::PUBLISH
    }
}

impl Publish {

    const PACKET_TYPE: u8 = 0b00110000;
    const DUP_FLAG_MASK: u8 = 0b00001000;
    const RETAIN_FLAG_MASK: u8 = 0b00000001;
    const QOS_MASK: u8 = 0b00000110;

    /// Creates a new Publish packet using sane defaults for everything but the supplied values.
    /// [Publish] doesn't implement `Default` primarily because a "meaningful" topic name is a must.
    pub fn new(topic_name: String, payload: Vec<u8>) -> Self {
        Self {
            dup:false,
            qos_level: QoS::AtMostOnce,
            retain: false,
            topic_name,
            packet_identifier: None,
            properties: None,
            payload,
        }
    }
}

impl From<Publish> for Vec<u8> {
    fn from(publish: Publish) -> Self {
        let mut result = Vec::new();
        
        let mut first_byte = Publish::PACKET_TYPE;
        if publish.dup {
            first_byte |= Publish::DUP_FLAG_MASK;
        }

        let qos: u8 = publish.qos_level.into();
        // shift qos bits to match their alignment in the resulting byte 
        // and OR them to the resulting byte
        first_byte |= qos << 1;

        if publish.retain {
            first_byte |= Publish::RETAIN_FLAG_MASK;
        }
        result.push(first_byte);

        result.append(&mut UTF8String::from(publish.topic_name.as_str()).into());

        if qos > 0 {
            if let Some(pid) = publish.packet_identifier {
                super::push_be_u16(pid, &mut result)
            } else {
                // FIXME this should include a check if the topic alias property is set
                // also it could use some error handling
                super::push_be_u16(0, &mut result)
            }
        }

        match publish.properties {
            Some(p) => result.append(&mut p.into()),
            None => result.push(0),
        }

        let mut payload = publish.payload;
        result.append(&mut payload);

        super::calculate_and_insert_length(&mut result);

        result
    }
}

impl TryFrom<&[u8]> for Publish {
    type Error = MqttError;

    fn try_from(src: &[u8]) -> Result<Self, Self::Error> {
        let mut cursor = 0;
        let packet_type = src[cursor] & Self::PACKET_TYPE;
        if packet_type != Self::PACKET_TYPE {
            return Err(MqttError::MalformedPacket(
                format!("Packet type is not a CONNECT packet: {:b}", packet_type)))
        }
        let dup = 1 == src[cursor] | Self::DUP_FLAG_MASK;
        let retain = 1 == src[cursor] | Self::RETAIN_FLAG_MASK;

        let qos_level = QoS::try_from((src[cursor] & Self::QOS_MASK) >> 1)?;
        cursor += 1;

        let remain_len = remaining_length(&src[cursor..])?;
        cursor += remain_len.encoded_len();
        let mut payload_len = remain_len.value as usize;

        // topic name
        /* TODO!
The Topic Name in the PUBLISH packet MUST NOT contain wildcard characters [MQTT-3.3.2-2].
1536
1537
1538 according to the matching process defined in section 4.7 [MQTT-3.3.2-3].
1539 However, as the Server is permitted to map the Topic Name to another name, it might not be the same as
1540 the Topic Name in the original PUBLISH packet.
1541
1542 To reduce the size of the PUBLISH packet the sender can use a Topic Alias. The Topic Alias is described
1543 in section 3.3.2.3.4. It is a Protocol Error if the Topic Name is zero length and there is no Topic Alias.
1544        
        */
        let topic_name_res = UTF8String::try_from(&src[cursor..])?;
        cursor += topic_name_res.encoded_len();
        payload_len -= topic_name_res.encoded_len();

        let topic_name = match topic_name_res.value {
            Some(v) => v,
            None => String::new(),
        };

        // packet ident
        // only present in case QoS is > 0
        let packet_identifier = match qos_level {
            QoS::AtMostOnce => None,
            _=> {
                let pid = super::u16_from_be_bytes(&src[cursor..cursor + 2])?;
                cursor += pid.encoded_len();
                payload_len -= pid.encoded_len();
                Some(pid)
            },
        };

        // properties
        let prop_res: DecodingResult<PublishProperties> = PublishProperties::decode(&src[cursor..])?;
        cursor += prop_res.bytes_read();
        payload_len -= prop_res.bytes_read();

        // payload
        let payload: Vec<u8> = src[cursor..cursor + payload_len].to_vec();

        Ok(Self {
            dup,
            qos_level,
            retain,
            topic_name,
            packet_identifier,
            properties: prop_res.value(),
            payload,
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn encode_and_decode() {
        let publish = test_packet();
        let topic_name = publish.topic_name.clone();
        
        let encoded: Vec<u8> = publish.into();
        let decoded = Publish::try_from(&encoded[..]).unwrap();

        assert_eq!(topic_name, decoded.topic_name);
    }

    /// the simplest form of a PUBLISH packet with just a topic and payload, no DUP, Qos 0, no retain, no properties
    #[test]
    fn encode() {
        let packet: Vec<u8> = test_packet().into();
        let expect: Vec<u8> = vec![48,40,0,15,115,111,109,101,47,116,111,112,105,99,47,110,97,109,101,0,123,34,115,111,109,101,34,58,49,44,34,102,111,111,34,58,34,98,97,114,34,125];
        assert_eq!(expect, packet);
    }
    
    #[test]
    fn encode_packet_id() {
        let mut publish = test_packet();
        publish.qos_level = QoS::AtLeastOnce;
        publish.packet_identifier = Some(8123);
        let packet: Vec<u8> = publish.into();
        let expect: Vec<u8> = vec![
            0b00110010, // qos bits set to 01
            42, 0, 15, 115, 111, 109, 101, 47, 116, 111, 112, 105, 99, 47, 110, 97, 109, 101,
            31, 187, // 8123 as big endian u16: 0001_1111, 1011_1011
            0, // props len
            123, 34, 115, 111, 109, 101, 34, 58, 49, 44, 34, 102, 111, 111, 34, 58, 34, 98, 97, 114, 34, 125 // payload
        ];
        
        assert_eq!(expect, packet);        
    }

    /// ensures that an assigned packet id is ignored during encoding if QoS is 0
    #[test]
    fn encode_ignore_packet_id() {
        let mut publish = test_packet();
        publish.packet_identifier = Some(8123);
        let packet: Vec<u8> = publish.into();
        let expect: Vec<u8> = vec![
            48, 40, 0, 15, 115, 111, 109, 101, 47, 116, 111, 112, 105, 99, 47, 110, 97, 109, 101,
            // packet id would be here
            0, // props len
            123, 34, 115, 111, 109, 101, 34, 58, 49, 44, 34, 102, 111, 111, 34, 58, 34, 98, 97, 114, 34, 125 // payload
        ];
        
        assert_eq!(expect, packet);
    }

    #[test]
    fn decode() {
        let msg: Vec<u8> = vec![48, 20, 0, 11, 47, 115, 111, 109, 101, 47, 116, 111, 112, 105, 99, 0, 115, 101, 114, 118, 117, 115];
        let publ = Publish::try_from(&msg[..]).unwrap();
        assert_eq!(false, publ.dup);
        assert!(publ.packet_identifier.is_none());
        assert!(publ.properties.is_none());
        assert_eq!(String::from("/some/topic"), publ.topic_name);
        assert_eq!(String::from("servus"), String::from_utf8(publ.payload).unwrap());
    }

    #[test]
    fn decode_wrong_packet_type() {
        let vec: Vec<u8> = vec![0b01010101];
        let res = Publish::try_from(&vec[..]);
        assert!(res.is_err());
    }

    #[test]
    fn encode_first_byte() {
        do_encode_first_byte(false, false, None, 0b00110000);
        do_encode_first_byte(true, false, None, 0b00111000);
        do_encode_first_byte(true, true, None, 0b00111001);
        do_encode_first_byte(false, false, Some(QoS::AtLeastOnce), 0b00110010);
        do_encode_first_byte(false, false, Some(QoS::ExactlyOnce), 0b00110100);
    }

    #[test]
    fn encode_properties() {
        let empty: PublishProperties = PublishProperties::default();
        let vempty: Vec<u8> = empty.into();
        assert_eq!(vec![0_u8], vempty);

        let mut props: PublishProperties = PublishProperties::default();
        props.payload_format_indicator = Some(true);
        props.user_property.insert("debug".to_string(), "true".to_string());
        props.topic_alias = Some(334);

        let expect: Vec<u8> = vec![19,1,1,35,1,78,38,0,5,100,101,98,117,103,0,4,116,114,117,101];
        let actual: Vec<u8> = props.into();
        assert_eq!(expect, actual);
    }

    /// another example from a 'real' mqtt broker
    #[test]
    fn decode_qos_1() {
        let msg: Vec<u8> = vec![48,43,0,11,47,115,111,109,101,47,116,111,112,105,99,0,123,34,104,101,112,112,34,58,34,115,99,104,105,110,103,34,44,34,99,105,97,111,34,58,116,114,117,101,125];
        let publ = Publish::try_from(&msg[..]);
        println!("{:?}", publ);
    }

    fn test_packet() -> Publish {
        Publish::new("some/topic/name".into(), r#"{"some":1,"foo":"bar"}"#.to_string().into_bytes())
    }

    fn do_encode_first_byte(dup: bool, retain: bool, qos: Option<QoS>, expected: u8) {
        let mut publish = Publish::new("".into(), vec![]);
        publish.dup = dup;
        publish.retain = retain;
        if let Some(q) = qos {
            publish.qos_level = q
        }
        let vec: Vec<u8> = publish.into();
        assert_eq!(expected, vec[0]);
    }
}
