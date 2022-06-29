use crate::{VariableByteInteger, types::ReasonCode};

use super::{MqttControlPacket, PacketType};

/// CONNACK
/// 
/// Fixed header (packet type 2 | reserved 0)
/// 0010 0000
/// remaining length
/// 
/// Variable Header
///     ack flags (1 byte) [bit 0 = session present), all others = 0]
///     reason code (1 byte)
///     Properties:
///         length: VBI
///         session expiry interval
///         receive max
///         max qos
///         retain available
///         max packet size
///         assigned client id
///         topic alias max
///         reason string
///         user property*
///         wildcard subscription available
///         subscription ids available
///         shared sub available
///         server keep alive
///         response info
///         server reference
///         auth method
///         auth data
/// 
/// NO payload
#[derive(Debug)]
pub struct ConnackPacket {
    pub session_present: bool,
    pub reason_code: ReasonCode,

}

impl From<&[u8]> for ConnackPacket {
    fn from(src: &[u8]) -> Self {
        // FIXME this really should be try_into so we can use results and error handling
        if src[0] != 32 {
            println!("WARNING: Connack packet started with wrong packet type: {}", src[0]);
        }

        let remaining_length = VariableByteInteger::from(&src[1..5]);

        // the index where the Variable Header begins
        let mut index = (1 + remaining_length.bytes_used()) as usize;
        
        // TODO should we actually do something with the session present flag if it is set? check the spec
        let session_present = src[index] != 0;

        index += 1;
        let reason_code = ReasonCode::try_from(src[index]).unwrap();

        // FIXME read the rest, duh!
        /*
        index += 1;
        let props_len = src[index];
        println!("when I grow up I will process {} bytes of properties!", props_len); 
        */

        ConnackPacket { session_present, reason_code }
    }
}

impl Into<Vec<u8>> for ConnackPacket {
    fn into(self) -> Vec<u8> {
        let mut packet: Vec<u8> = Vec::new();
        packet.push(32);
        packet.push(3); // FIXME calculate the remaining length
        packet.push(self.session_present.into());
        packet.push(self.reason_code.into());
        packet.push(0); // FIXME properties go here...
        packet
    }
}

impl MqttControlPacket for ConnackPacket {
    
    fn packet_type() -> PacketType {
        PacketType::CONNACK
    }
    
    fn payload_requirement() -> crate::types::YesNoMaybe {
        crate::types::YesNoMaybe::None
    }
}

#[cfg(test)]
mod tests {

    use std::vec;

    use super::*;

    #[test]
    fn decode() {
        // normal, clean session and reason code 0
        run_decode(
            &vec![32, 12, 0, 0, 9, 19, 255, 255, 34, 0, 10, 33, 0, 20], 
            false, 
            ReasonCode::Success);

        // session present flag set
        run_decode(
            &vec![32, 3, 1, 0, 0], 
            true, 
            ReasonCode::Success);

        // reason codes...
        run_decode(
            &vec![32, 3, 0, 0x80, 0], 
            false, 
            ReasonCode::UnspecifiedError);

        run_decode(
            &vec![32, 3, 1, 0x8C, 0], // bad authentication
            true, 
            ReasonCode::BadAuthenticationMethod);
    }

    #[test]
    fn encode() {
        let connack = ConnackPacket { session_present: false, reason_code: ReasonCode::Success };
        let bin: Vec<u8> = connack.into();
        let expected = vec![32, 3, 0, 0, 0];
        assert_eq!(expected, bin);
    }

    fn run_decode(binary: &[u8], session_present: bool, reason_code: ReasonCode) {
        let connack = ConnackPacket::from(binary);

        assert_eq!(session_present, connack.session_present);
        assert_eq!(reason_code, connack.reason_code);
    }
}