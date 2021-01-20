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
}

impl Default for Config {
    fn default() -> Self {
        Config {
            service_port: default_service_port(),
            time: None,
            x11: None,
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
}

impl Default for X11Config {
    fn default() -> Self {
        X11Config {
            display: default_display(),
        }
    }
}
