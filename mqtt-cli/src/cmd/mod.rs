pub mod publish;
pub mod subscribe;

use clap::{Parser, Subcommand};

use self::{subscribe::SubscribeCmd, publish::PublishCmd};

#[derive(Debug, Parser)]
#[command(name = "mqtt-cli", about = "MQTT command line client", disable_help_flag = true)]
pub struct MqttCli {

    /// command to run
    #[command(subcommand)]
    pub command: Command,

    /// turns on debug logging
    #[arg(global = true, short, long)]
    pub verbose: bool,

    /// optional server host name, defaults to `localhost`
    #[arg(global = true, short, long)]
    pub host: Option<String>,

    /// optional port number, defaults to `1883` (TODO: `8883` when using TLS)
    #[arg(global = true, short, long)]
    pub port: Option<u16>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// publishes to a broker
    Pub(PublishCmd),

    /// subscribes to a topic
    Sub(SubscribeCmd),
}