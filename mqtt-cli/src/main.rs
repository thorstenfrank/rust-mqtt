//! A simple MQTT command-line client.

mod client;
mod cmd;
mod session;

use clap::Parser;
use cmd::{Command, MqttCli};
use mqtt::error::MqttError;
use session::Session;

type CmdResult = Result<(), MqttError>;

fn main() -> CmdResult {
    let args = MqttCli::parse();

    let host = args.host.unwrap_or(String::from("localhost"));
    let port = args.port.unwrap_or(1883);

    let session = Session::new(args.verbose, (host, port));

    match args.command {
        Command::Pub(publ) => publ.execute(session),
        Command::Sub(sub) => sub.execute(session),
    }
}
