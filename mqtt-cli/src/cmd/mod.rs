pub mod publish;
pub mod subscribe;

use clap::{Subcommand, Parser};

use self::{subscribe::SubscribeCmd, publish::PublishCmd};

#[derive(Debug, Parser)]
pub struct MqttCli {

    /// command to run
    #[clap(subcommand)]
    pub command: Command,

    /// turns on debug logging
    #[clap(global = true, short, long, parse(from_flag))]
    pub verbose: bool,

    /// optional server host name, defaults to `localhost`
    #[clap(global = true, short, long)]
    pub host: Option<String>,

    /// optional port number, defaults to `1883` (TODO: `8883` when using TLS)
    #[clap(global = true, short, long)]
    pub port: Option<u16>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// publishes to a broker
    Pub(PublishCmd),

    /// subscribes to a topic
    Sub(SubscribeCmd),
}