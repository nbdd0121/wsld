use super::config::TimeConfig;
use super::vmsocket::VmSocket;
use super::CONFIG;

use std::io::{Error, ErrorKind};
use std::time::{Duration, SystemTime};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

async fn sync_time() -> std::io::Result<()> {
    let mut stream = VmSocket::connect(CONFIG.service_port).await?;
    let start = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    stream.write_all(b"time").await?;
    let time = Duration::from_micros(stream.read_u64().await?);
    let end = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();

    // This operation will take some time (~0.1ms), so we adjust the time
    // based on time elapsed (similar to NTP).
    let delay = match end.checked_sub(start) {
        Some(delay) => delay / 2,
        // Time has jumped, so someone has changed time. Assume
        // this is user-initiated time sync so return Ok.
        None => return Ok(()),
    };
    let time = time + delay;
    let diff = time.as_micros() as i64 - end.as_micros() as i64;

    // Set system time if difference is larger than 0.5sec, set time
    // adjtime only works if difference is within 0.5sec
    let set = diff.abs() >= 500_000;

    let ret = if set {
        let timeval = libc::timeval {
            tv_sec: time.as_secs() as _,
            tv_usec: time.subsec_micros() as _,
        };

        unsafe { libc::settimeofday(&timeval, std::ptr::null()) }
    } else {
        let mut timex: libc::timex = unsafe { std::mem::zeroed() };
        timex.modes = libc::ADJ_OFFSET_SINGLESHOT;
        timex.offset = diff;

        unsafe { libc::adjtimex(&mut timex) }
    };

    let result = if ret >= 0 {
        Ok(())
    } else {
        Err(Error::last_os_error())
    };

    let time_st = humantime::format_rfc3339_micros(SystemTime::UNIX_EPOCH + time);
    let diff_str = format!(
        "{}{}",
        if diff < 0 { "-" } else { "" },
        humantime::format_duration(Duration::from_micros(diff.abs() as u64))
    );
    eprintln!(
        "Received time {}, clock off by {}, {}",
        time_st,
        diff_str,
        if set { "set" } else { "adjust" }
    );

    if let Err(ref err) = result {
        if let ErrorKind::PermissionDenied = err.kind() {
            eprintln!("Cannot set time, run with root or set CAP_SET_TIME");
        }
    }

    result
}

pub async fn timekeeper(config: &'static TimeConfig) -> std::io::Result<()> {
    loop {
        sync_time().await?;
        tokio::time::sleep(config.interval).await;
    }
}
