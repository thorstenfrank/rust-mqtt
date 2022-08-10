use clap::Parser;
use mqtt::types::QoS;
use crate::{Session, client::Client, CmdResult};

#[derive(Debug, Parser)]
pub struct SubscribeCmd {
    /// Topic pattern to subscribe to, may include wildcards (`+` or `#`).
    #[clap(short, long)]
    topic: String,

    /// Quality of Service level. 1 or 2. 0 is the default, no need to expliclty specify in that case.
    #[clap(short, long)]
    qos: Option<u8>,
}

impl SubscribeCmd {

    pub fn execute(&self, session: Session) -> CmdResult {
        let mut topic = mqtt::packet::TopicFilter::new(self.topic.clone());
        if let Some(qos) = self.qos {
            topic.maximum_qos = QoS::try_from(qos)?;
        }

        let subscribe = mqtt::packet::Subscribe{
            packet_identifier: session.packet_identifier(),
            properties: None,
            topic_filter: vec![topic],
        };

        let mut client = Client::connect(session)?;

        client.subscribe(subscribe)?;

        Ok(())
    }
}