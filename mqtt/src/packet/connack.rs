use std::collections::HashMap;

use mqtt_derive::MqttProperties;

use crate::{error::MqttError, types::{MqttDataType, ReasonCode, QoS, VariableByteInteger}};

use super::{MqttControlPacket, PacketType, Decodeable, DecodingResult};

const FIRST_BYTE: u8 = 0b00100000;
/// A `CONNACK` MQTT control packet.
#[derive(Debug)]
pub struct Connack {

    /// Whether this connect/connack exchange resumes an existing session or starts a new one.
    pub session_present: bool,

    /// Indicates whether the connection attempt was successful, and if not why.
    /// [Anything above 0x80 is an error](crate::types::ReasonCode::is_err()).
    pub reason_code: ReasonCode,

    /// Optional properties sent by the server.
    pub properties: Option<ConnackProperties>,
}

/// Sums up all properties a server may send.
#[derive(Debug, MqttProperties)]
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
    pub retain_available: Option<bool>,

    /// Maximum size in number of bytes the server is willing to accept.
    pub maximum_packet_size: Option<u32>,

    /// Server-issued in cases where the client does not specify its own ID with the `CONNECT` packet.
    pub assigned_client_identifier: Option<String>,

    /// Maximum number of numeric aliases the server allows.
    pub topic_alias_maximum: Option<u16>,

    /// Additional human-readable information from the server.
    pub reason_string: Option<String>,

    /// Generic key-value properties.
    pub user_property: HashMap<String, String>,

    /// 
    pub wildcard_subscription_available: Option<bool>,

    ///
    pub subscription_identifier_available: Option<bool>,

    ///
    pub shared_subscription_available: Option<bool>,

    ///
    pub server_keep_alive: Option<u16>,

    /// Application-level instructions on how to build the response topic such as the base of the topic tree.
    pub response_information: Option<String>,

    /// Used in conjunction with [reason code 0x9C](crate::types::ReasonCode::UseAnotherServer) to define a different
    /// server for the client to use.
    pub server_reference: Option<String>,

    /// Application-specific auth method
    pub authentication_method: Option<String>,

    /// Application-specific auth data
    pub authentication_data: Option<Vec<u8>>,
}

impl TryFrom<&[u8]> for Connack {
    
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

        let prop_res: DecodingResult<ConnackProperties> = ConnackProperties::decode(&src[index..])?;

        Ok(Connack { session_present, reason_code, properties: prop_res.value() })
    }
}

impl From<Connack> for Vec<u8> {
    
    fn from(connack: Connack) -> Self {
        let mut packet: Vec<u8> = Vec::new();

        packet.push(FIRST_BYTE);
        packet.push(connack.session_present.into());
        packet.push(connack.reason_code.into());
        
        match connack.properties {
            Some(props) => {
                packet.append(&mut props.into());
            },
            None => {
                packet.push(0)
            },
        };

        super::calculate_and_insert_length(&mut packet);

        packet
    }
}

impl MqttControlPacket<'_> for Connack {
    
    fn packet_type() -> PacketType {
        PacketType::CONNACK
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
        assert_eq!(Some(10_u16), props.topic_alias_maximum);
        assert_eq!(Some(20_u16), props.receive_maximum);
        assert_eq!(Some(false), props.shared_subscription_available);

        Ok(())
    }

    #[test]
    fn encode() {
        let connack = Connack { session_present: false, reason_code: ReasonCode::Success, properties: None };
        let bin: Vec<u8> = connack.into();
        let expected = vec![32, 3, 0, 0, 0];
        assert_eq!(expected, bin);
    }

    #[test]
    fn encode_with_properties() {
        let mut properties = ConnackProperties::default();
        properties.assigned_client_identifier = Some("generated-123456".into());
        properties.server_keep_alive = Some(135);
        let connack = Connack { session_present: true, reason_code: ReasonCode::Success, properties: Some(properties) };
        let actual: Vec<u8> = connack.into();
        let expect: Vec<u8> = vec![32, 25, 1, 0, 22, 18, 0, 16, 103, 101, 110, 101, 114, 97, 116, 101, 100, 45, 49, 50, 51, 52, 53, 54, 19, 0, 135];
        assert_eq!(expect, actual);
    }

    fn run_decode(binary: &[u8], session_present: bool, reason_code: ReasonCode, expect_properties: bool) -> Result<Connack, MqttError> {
        let connack = Connack::try_from(binary)?;

        assert_eq!(session_present, connack.session_present);
        assert_eq!(reason_code, connack.reason_code);

        match expect_properties {
            true => assert!(connack.properties.is_some()),
            false => assert!(connack.properties.is_none()),
        };

        Ok(connack)
    }

    #[test]
    fn property_defaults() {
        let p: ConnackProperties = ConnackProperties::default();
        
        assert!(p.retain_available.is_none());
        assert!(p.shared_subscription_available.is_none());
        assert!(p.subscription_identifier_available.is_none());
        assert!(p.wildcard_subscription_available.is_none());
        assert!(p.assigned_client_identifier.is_none());
        assert!(p.authentication_data.is_none());
        assert!(p.authentication_method.is_none());
        assert!(p.reason_string.is_none());
        assert!(p.receive_maximum.is_none());
        assert!(p.maximum_qos.is_none());
    }
}