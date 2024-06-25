use std::fs::OpenOptions;
use std::io::Result;
use std::io::Write;
use std::process::Command;
use std::thread;
use std::time::{Duration, SystemTime};

pub fn extract_service_logs(service: &str, since: &str) -> Result<String> {
    let output = Command::new("journalctl")
        .arg("-u")
        .arg(service)
        .arg("--since")
        .arg(since)
        .arg("--no-pager")
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(std::io::Error::new(std::io::ErrorKind::Other, stderr));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

pub fn extract_kernel_logs(since: &str) -> Result<String> {
    let output = Command::new("journalctl")
        .arg("-k")
        .arg("--since")
        .arg(since)
        .arg("--no-pager")
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(std::io::Error::new(std::io::ErrorKind::Other, stderr));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

pub fn spawn_log_writer(service: &str, log_file: &str) -> Result<()> {
    let log_file_path = log_file.to_string();
    let service = service.to_string();
    thread::spawn(move || {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&log_file_path)
            .expect("Failed to open log file");
        let mut last_fetch_time = SystemTime::now();
        loop {
            let since = format!("{:?}", last_fetch_time);
            match extract_service_logs(&service, &since) {
                Ok(logs) => {
                    if !logs.trim().is_empty() {
                        if let Err(e) = writeln!(file, "{}", logs) {
                            eprintln!("Failed to write logs to file: {}", e);
                        }
                        last_fetch_time = SystemTime::now();
                    }
                }
                Err(e) => eprintln!("Failed to fetch logs: {} {}", e, service),
            }
            thread::sleep(Duration::from_secs(1));
        }
    });
    Ok(())
}
