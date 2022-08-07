use clap::Parser;
use crate::{Session, client::Client};

#[derive(Debug, Parser)]
pub struct Subscribe {
    /// Topic to subscribe to
    #[clap(short, long)]
    topic: String,
}

impl Subscribe {

    pub fn execute(&self, session: Session) {
        let topic = mqtt::packet::TopicFilter::new(self.topic.clone());

        let subscribe = mqtt::packet::Subscribe{
            packet_identifier: session.packet_identifier(),
            properties: None,
            topic_filter: vec![topic],
        };

        let mut client = Client::connect(session).unwrap_or_else(|err| {
            panic!("{:?}", err)
        });

        client.subscribe(subscribe).unwrap_or_else(|err| {
            panic!("{:?}", err)
        });
    }
}