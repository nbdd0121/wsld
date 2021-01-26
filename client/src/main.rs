mod config;
mod time;
mod util;
mod vmsocket;
mod x11;
mod x11socket;

use config::Config;
use vmsocket::VmSocket;

use once_cell::sync::Lazy;
use std::io::ErrorKind;
use std::process::exit;
use tokio::io::AsyncWriteExt;

static CONFIG: Lazy<Config> = Lazy::new(|| {
    let args: Vec<_> = std::env::args().collect();
    let (config_path, home) = if args.len() == 2 {
        ({ args }.swap_remove(1).into(), false)
    } else {
        let mut config_path = dirs::home_dir().unwrap_or_else(|| {
            eprintln!("cannot find home dir");
            exit(1);
        });
        config_path.push(".wsld.toml");
        (config_path, true)
    };
    let config_file = match std::fs::read(&config_path) {
        Ok(f) => f,
        Err(err) if err.kind() == ErrorKind::NotFound && home => {
            // If .wsld.toml isn't there, do its name: X11 forwarding
            return Config {
                x11: Some(Default::default()),
                ..Default::default()
            };
        }
        Err(err) => {
            eprintln!("cannot read {:?}: {}", config_path, err);
            exit(1);
        }
    };
    toml::from_slice(&config_file).unwrap_or_else(|err| {
        eprintln!("invalid config file: {}", err);
        exit(1);
    })
});

async fn wait_host_up() -> std::io::Result<()> {
    let mut retry = 5usize;
    loop {
        match VmSocket::connect(CONFIG.service_port).await {
            Ok(mut stream) => {
                stream.write_all(b"noop").await?;
                return Ok(());
            }
            Err(err) if err.kind() == ErrorKind::TimedOut => {
                if retry == 0 {
                    return Err(err);
                }
                retry -= 1;
            }
            Err(err) => return Err(err),
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    Lazy::force(&CONFIG);

    if let Err(err) = wait_host_up().await {
        eprintln!("Cannot connect to wsldhost: {}", err);
        return;
    }

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
