use clap::Parser;
use mqtt::{types::QoS, error::MqttError};
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
        client.listen();

        println!();
        println!("##################################################");
        println!("now listening for messages, press 'ENTER' to quit");
        println!("##################################################");
        println!();
        
        match std::io::stdin().read_line(&mut String::new()) {
            Ok(_) => client.disconnect(),
            Err(e) => Err(MqttError::Message(format!("error reading user input: {:?}", e))),
        }
    }
}