use clap::Parser;
use mqtt::types::QoS;

use crate::{client::Client, Session, CmdResult};

#[derive(Debug, Parser)]
pub struct PublishCmd {
    /// Topic to publish to
    #[arg(short, long)]
    topic: String,

    /// message payload
    #[arg(short, long)]
    message: String,

    /// Quality of Service level. 0 (at most once), 1 (at least once), 2 (exactly once)
    #[arg(short, long)]
    qos: Option<u8>,
}

impl PublishCmd {

    pub fn execute(&self, session: Session) -> CmdResult {
        let mut publish = mqtt::packet::Publish::new(
            self.topic.clone(), 
            self.message.clone().into_bytes());

        if let Some(qos) = self.qos {
            publish.qos_level = QoS::try_from(qos)?;
            if qos == 1 || qos == 2 {
                publish.packet_identifier = Some(session.packet_identifier())
            }
        }
        
        let mut client = Client::connect(session)?;
        
        client.publish( publish)?;

        client.disconnect()?;

        Ok(())
    }
}