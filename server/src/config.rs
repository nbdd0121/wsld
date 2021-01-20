use std::io::{Error, ErrorKind};
use structopt::StructOpt;
use uuid::Uuid;

fn parse_uuid(str: &str) -> std::io::Result<Uuid> {
    str.parse()
        .map_err(|err| Error::new(ErrorKind::InvalidInput, format!("Invalid UUID: {}", err)))
}

#[derive(Debug, StructOpt)]
#[structopt(name = "wsldhost")]
pub struct Config {
    #[structopt(short, long)]
    pub daemon: bool,

    #[structopt(short = "p", long, default_value = "6000")]
    pub service_port: u32,

    #[structopt(name = "VMID", parse(try_from_str = parse_uuid))]
    pub vmid: Option<Uuid>,

    #[structopt(flatten)]
    pub x11: X11Config,
}

#[derive(Debug, StructOpt)]
pub struct X11Config {
    #[structopt(long, default_value = "localhost:6000")]
    pub display: String,
}
