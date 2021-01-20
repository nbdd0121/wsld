mod config;
mod time;
mod vmsocket;
mod x11;
mod x11socket;

use config::Config;

use once_cell::sync::Lazy;
use std::process::exit;

static CONFIG: Lazy<Config> = Lazy::new(|| {
    let mut config_path = dirs::home_dir().unwrap_or_else(|| {
        eprintln!("cannot find home dir");
        exit(1);
    });
    config_path.push(".wsld.toml");
    let config_file = std::fs::read(config_path).unwrap_or_else(|err| {
        eprintln!("cannot read ~/.wsld.toml: {}", err);
        exit(1);
    });
    toml::from_slice(&config_file).unwrap_or_else(|err| {
        eprintln!("invalid config file: {}", err);
        exit(1);
    })
});

#[tokio::main(flavor = "current_thread")]
async fn main() {
    Lazy::force(&CONFIG);
    let mut tasks = Vec::new();

    if let Some(config) = &CONFIG.time {
        tasks.push(tokio::task::spawn(async move {
            let err = time::timekeeper(config).await.unwrap_err();
            eprintln!("Timekeeper error: {}", err);
        }));
    }

    if let Some(config) = &CONFIG.x11 {
        tasks.push(tokio::task::spawn(async move {
            if let Err(err) = x11::x11_forward(config).await {
                eprintln!("Failed to listen: {}", err);
            }
        }));
    }

    for task in tasks {
        let _ = task.await;
    }
}
