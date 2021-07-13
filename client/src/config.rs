use serde::{Deserialize, Serialize};
use std::time::Duration;

fn default_service_port() -> u32 {
    6000
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    #[serde(default = "default_service_port")]
    pub service_port: u32,

    #[serde(default)]
    pub time: Option<TimeConfig>,

    #[serde(default)]
    pub x11: Option<X11Config>,

    #[serde(default)]
    pub tcp_forward: Option<TcpForwardConfig>,

    #[serde(default)]
    pub ssh_agent: Option<SshAgentConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            service_port: default_service_port(),
            time: None,
            x11: None,
            tcp_forward: None,
            ssh_agent: None,
        }
    }
}

fn default_interval() -> Duration {
    // Every 10 minutes
    Duration::from_secs(600)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TimeConfig {
    #[serde(default = "default_interval")]
    #[serde(with = "humantime_serde")]
    pub interval: Duration,
}

fn default_display() -> u32 {
    // Display :0
    0
}

#[derive(Serialize, Deserialize, Debug)]
pub struct X11Config {
    #[serde(default = "default_display")]
    pub display: u32,

    #[serde(default)]
    pub force: bool,
}

impl Default for X11Config {
    fn default() -> Self {
        X11Config {
            display: default_display(),
            force: false,
        }
    }
}

fn default_tcp_service_port() -> u16 {
    // Don't use 6000 to avoid clash with X
    6001
}

fn default_iptables_cmd() -> String {
    "sudo iptables-legacy".to_owned()
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TcpForwardConfig {
    #[serde(default = "default_tcp_service_port")]
    pub service_port: u16,

    #[serde(default = "default_iptables_cmd")]
    pub iptables_cmd: String,

    pub ports: Vec<u16>,
}

fn default_ssh_auth_sock() -> String {
    "/tmp/.wsld/ssh_auth_sock".to_owned()
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SshAgentConfig {
    #[serde(default = "default_ssh_auth_sock")]
    pub ssh_auth_sock: String,
}
