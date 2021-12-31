use clap::Parser;
use std::io::{Error, ErrorKind};
use uuid::Uuid;

fn parse_uuid(str: &str) -> std::io::Result<Uuid> {
    str.parse()
        .map_err(|err| Error::new(ErrorKind::InvalidInput, format!("Invalid UUID: {}", err)))
}

#[derive(Debug, Parser)]
#[clap(name = "wsldhost")]
pub struct Config {
    #[clap(short, long)]
    pub daemon: bool,

    #[clap(short = 'p', long, default_value = "6000")]
    pub service_port: u32,

    #[clap(name = "VMID", parse(try_from_str = parse_uuid))]
    pub vmid: Option<Uuid>,

    #[clap(flatten)]
    pub x11: X11Config,
}

#[derive(Debug, Parser)]
pub struct X11Config {
    #[clap(long, default_value = "127.0.0.1:6000")]
    pub display: String,
}
