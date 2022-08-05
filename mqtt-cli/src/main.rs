mod client;
mod publish;
mod subscribe;

//const DEFAULT_PORT: u16 = 1883;
use clap::{Subcommand, Parser};
use crate::{publish::Publish, subscribe::Subscribe};

#[derive(Debug, Parser)]
struct Args {

    /// command to run
    #[clap(subcommand)]
    command: Command,

    /// turns on debug logging
    #[clap(global = true, short, long, parse(from_flag))]
    verbose: bool,

    /// optional server host name, defaults to `localhost`
    #[clap(global = true, short, long)]
    host: Option<String>,

    /// optional port number, defaults to `1883` (TODO: `8883` when using TLS)
    #[clap(global = true, short, long)]
    port: Option<u16>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// publishes to a broker
    Pub(Publish),

    /// subscribes to a topic
    Sub(Subscribe),
}

pub struct Session {
    debug: bool,
    addr: (String, u16),
}

fn main() {
    let args = Args::parse();

    let host = args.host.unwrap_or(String::from("localhost"));
    let port = args.port.unwrap_or(1883);

    let session = Session {debug: args.verbose, addr: (host, port)};

    match args.command {
        Command::Pub(publ) => publ.execute(session),
        Command::Sub(sub) => sub.execute(session),
    }
}