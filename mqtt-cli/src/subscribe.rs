use clap::Parser;

use crate::Session;

#[derive(Debug, Parser)]
pub struct Subscribe {
    /// Topic to subscribe to
    #[clap(short, long)]
    topic: String,
}

impl Subscribe {

    pub fn execute(&self, _session: Session) {
        println!("Subscribing to topic [{}]", self.topic);
        println!("!!! TO BE IMPLEMENTED !!!");
    }
}