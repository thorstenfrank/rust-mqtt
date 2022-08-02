use crate::error::MqttError;

use super::MqttDataType;

/// MQTT-3.2.2.2: Connect Reason Codes, a single byte numeric value.
/// Anything above 0x80 is considered an error. 
/// See the spec for details.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReasonCode {
    /// 0x00 (0)
    Success = 0x00,
    /// 0x01 (1)
    GrantedQoS1 = 0x01,
    /// 0x02 (2)
    GrantedQoS2 = 0x02,
    /// 0x04 (4)
    DisconnectWithWill = 0x04,
    /// 0x10 (16)
    NoMatchingSubscribers = 0x10,
    /// 0x11 (17)
    NoSubscriptionExisted = 0x11,
    /// 0x18 (24)
    ContinueAuthentication = 0x18,
    /// 0x19 (25)
    ReAuthenticate = 0x19,
    /// 0x80 (128)
    UnspecifiedError = 0x80,
    /// 0x81 (129)
    MalformedPacket = 0x81,
    /// 0x82 (130)
    ProtocolError = 0x82,
    /// 0x83 (131)
    ImplementationSpecificError = 0x83,
    /// 0x84 (132)
    UnsupportedProtocolVersion = 0x84,
    /// 0x85 (133)
    ClientIdentifierInvalid = 0x85,
    /// 0x86 (134)
    BadUserNameOrPassword = 0x86,
    /// 0x87 (135)
    NotAuthorized = 0x87,
    /// 0x88 (136)
    ServerUnavailable = 0x88,
    /// 0x89 (137)
    ServerBusy = 0x89,
    /// 0x8A (138)
    Banned = 0x8A,
    /// 0x8B (139)
    ServerShuttingDown = 0x8B,
    /// 0x8C (140)
    BadAuthenticationMethod = 0x8C,
    /// 0x8D (141)
    KeepAliveTimeout = 0x8D,
    /// 0x8E (142)
    SessionTakenOver = 0x8E,
    /// 0x8F (143)
    TopciFilterInvalid = 0x8F,
    /// 0x90 (144)
    TopicNameInvalid = 0x90,
    /// 0x91 (145)
    PacketIdentifierInUse = 0x91,
    /// 0x92 (146)
    PacketIdentifierNotFound = 0x92,
    /// 0x93 (147)
    ReceiveMaximumExceeded = 0x93,
    /// 0x94 (148)
    TopicAliasInvalid = 0x94,
    /// 0x95 (149)
    PacketTooLarge = 0x95,
    /// 0x96 (150)
    MessageRateToohigh = 0x96,
    /// 0x97 (151)
    QuotaExceeded = 0x97,
    /// 0x98 (152)
    AdministrativeAction = 0x98,
    /// 0x99 (153)
    PayloadFormatInvalid = 0x99,
    /// 0x9A (154)
    RetainNotSupported = 0x9A,
    /// 0x98 (155)
    QoSNotSupported = 0x9B,
    /// 0x9C (156)
    UseAnotherServer = 0x9C,
    /// 0x9D (157)
    ServerMoved = 0x9D,
    /// 0x9E (158)
    SharedSubscriptionsNotSupported = 0x9E,
    /// 0x9F (159)
    ConnectionRateExceeded = 0x9F,
    /// 0xA0 (160)
    MaximumConnectionTime = 0xA0,
    /// 0xA1 (161)
    SubscriptionIdentifiersNotSupported = 0xA1,
    /// 0xA2 (162)
    WildcardSubscriptionsNotSupported = 0xA2,
}

impl ReasonCode {

    /// Returns `true` if the reason code has a numeric value of 0x80 or higher.
    pub fn is_err(&self) -> bool {
        let num = *self as u8;
        num <= 128
    }
}

impl MqttDataType for ReasonCode {
    fn encoded_len(&self) -> usize {
        1
    }
}
impl From<ReasonCode> for u8 {
    fn from(reason_code: ReasonCode) -> Self {
        reason_code as u8
    }
}

impl TryFrom<u8> for ReasonCode {
    type Error = MqttError;

    /// Converts numeric values to a reason code enum, or returns an error if the code is undefined
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Success),
            1 => Ok(Self::GrantedQoS1),
            2 => Ok(Self::GrantedQoS2),
            4 => Ok(Self::DisconnectWithWill),
            16 => Ok(Self::NoMatchingSubscribers),
            17 => Ok(Self::NoSubscriptionExisted),
            24 => Ok(Self::ContinueAuthentication),
            25 => Ok(Self::ReAuthenticate),
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
            139 => Ok(Self::ServerShuttingDown),
            140 => Ok(Self::BadAuthenticationMethod),
            141 => Ok(Self::KeepAliveTimeout),
            142 => Ok(Self::SessionTakenOver),
            143 => Ok(Self::TopciFilterInvalid),
            144 => Ok(Self::TopicNameInvalid),
            145 => Ok(Self::PacketIdentifierInUse),
            146 => Ok(Self::PacketIdentifierNotFound),
            147 => Ok(Self::ReceiveMaximumExceeded),
            148 => Ok(Self::TopicAliasInvalid),
            149 => Ok(Self::PacketTooLarge),
            150 => Ok(Self::MessageRateToohigh),
            151 => Ok(Self::QuotaExceeded),
            152 => Ok(Self::AdministrativeAction),
            153 => Ok(Self::PayloadFormatInvalid),
            154 => Ok(Self::RetainNotSupported),
            155 => Ok(Self::QoSNotSupported),
            156 => Ok(Self::UseAnotherServer),
            157 => Ok(Self::ServerMoved),
            158 => Ok(Self::SharedSubscriptionsNotSupported),
            159 => Ok(Self::ConnectionRateExceeded),
            160 => Ok(Self::MaximumConnectionTime),
            161 => Ok(Self::SubscriptionIdentifiersNotSupported),
            162 => Ok(Self::WildcardSubscriptionsNotSupported),
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