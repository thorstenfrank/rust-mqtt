use crate::error::MqttError;

/// MQTT-3.2.2.2: Connect Reason Codes, a single byte numeric value.
/// If the server sends a CONNACK with a reason code >= 128, it MUST close the network connection.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReasonCode {
    Success = 0x00,
    UnspecifiedError = 0x80,
    MalformedPacket = 0x81,
    ProtocolError = 0x82,
    ImplementationSpecificError = 0x83,
    UnsupportedProtocolVersion = 0x84,
    ClientIdentifierInvalid = 0x85,
    BadUserNameOrPassword = 0x86,
    NotAuthorized = 0x87,
    ServerUnavailable = 0x88,
    ServerBusy = 0x89,
    Banned = 0x8A,
    BadAuthenticationMethod = 0x8C,
    TopicNameInvalid = 0x90,
    PacketTooLarge = 0x95,
    QuotaExceeded = 0x97,
    PayloadFormatInvalid = 0x99,
    RetainNotSupported = 0x9A,
    QoSNotSupported = 0x9B,
    UseAnotherServer = 0x9C,
    ServerMoved = 0x9D,
    ConnectionRateExceeded = 0x9F,
}

impl TryFrom<u8> for ReasonCode {
    type Error = MqttError;

    /// Converts numeric values to a reason code enum, or returns an error if the code is undefined
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Success),
            128 => Ok(Self::UnspecifiedError),
            129 => Ok(Self::MalformedPacket),
            130 => Ok(Self::ProtocolError),
            131 => Ok(Self::ImplementationSpecificError),
            132 => Ok(Self::UnsupportedProtocolVersion),
            133 => Ok(Self::ClientIdentifierInvalid),
            134 => Ok(Self::BadUserNameOrPassword),
            135 => Ok(Self::NotAuthorized),
            136 => Ok(Self::ServerUnavailable),
            137 => Ok(Self::ServerBusy),
            138 => Ok(Self::Banned),
            140 => Ok(Self::BadAuthenticationMethod),
            144 => Ok(Self::TopicNameInvalid),
            149 => Ok(Self::PacketTooLarge),
            151 => Ok(Self::QuotaExceeded),
            153 => Ok(Self::PayloadFormatInvalid),
            154 => Ok(Self::RetainNotSupported),
            155 => Ok(Self::QoSNotSupported),
            156 => Ok(Self::UseAnotherServer),
            157 => Ok(Self::ServerMoved),
            159 => Ok(Self::ConnectionRateExceeded),
            _=> Err(MqttError::Message(format!("Undefined Reason Code: {}", value))),
        }        
    }
}

#[cfg(test)]
mod tests {
    
    use super::*;

    #[test]
    fn test_reason_code_conversions() {
        assert_eq!(Ok(ReasonCode::Success), ReasonCode::try_from(0));
        assert_eq!(Ok(ReasonCode::ServerBusy), ReasonCode::try_from(137));
        assert_eq!(Ok(ReasonCode::TopicNameInvalid), ReasonCode::try_from(144));
        assert_eq!(Ok(ReasonCode::ProtocolError), ReasonCode::try_from(130));
        assert_eq!(Ok(ReasonCode::Banned), ReasonCode::try_from(138));
        
        let err1 = ReasonCode::try_from(0xFF);
        assert!(err1.is_err());
        assert_eq!(Some(MqttError::Message("Undefined Reason Code: 255".to_string())), err1.err());

        let err2 = ReasonCode::try_from(0xBA);
        assert!(err2.is_err());
        assert_eq!(Some(MqttError::Message("Undefined Reason Code: 186".to_string())), err2.err());
    }
}