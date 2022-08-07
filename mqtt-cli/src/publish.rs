use clap::Parser;
use mqtt::types::QoS;

use crate::{client::Client, Session};

#[derive(Debug, Parser)]
pub struct Publish {
    /// Topic to publish to
    #[clap(short, long)]
    topic: String,

    /// message payload
    #[clap(short, long)]
    message: String,

    /// Quality of Service level. 0 (at most once), 1 (at least once), 2 (exactly once)
    #[clap(short, long)]
    qos: Option<u8>,
}

impl Publish {

    pub fn execute(&self, session: Session) {
        let mut publish = mqtt::packet::Publish::new(self.topic.clone(), self.message.clone().into_bytes());
        if let Some(qos) = self.qos {
            publish.qos_level = QoS::try_from(qos).unwrap();
            if qos == 1 || qos == 2 {
                publish.packet_identifier = Some(session.packet_identifier())
            }
        }
        
        let mut client = Client::connect(session).unwrap_or_else(|err| {
            panic!("{:?}", err)
        });
        
        client.publish( publish).unwrap_or_else(|err| {
            panic!("{:?}", err)
        });

        client.disconnect().unwrap();
    }
}