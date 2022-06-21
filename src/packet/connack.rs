use super::{MqttControlPacket, PacketType};

pub struct ConnackPacket {

}

impl MqttControlPacket for ConnackPacket {
    
    fn packet_type() -> PacketType {
        PacketType::CONNACK
    }
    
}