use log::{error, info};
use log4rs;
use std::fs::OpenOptions;
use std::io::Result;
use std::io::Write;
use std::process::Command;
use std::thread;
use std::time::{Duration, SystemTime};

pub struct LogWriter {
    log_file_path: String,
}

impl LogWriter {
    pub fn new(log_file: &str, config_file: &str) -> Self {
        log4rs::init_file(config_file, Default::default()).unwrap();
        Self {
            log_file_path: log_file.to_string(),
        }
    }

    fn write_log(&self, logs: &str) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&self.log_file_path)?;
        if !logs.trim().is_empty() {
            writeln!(file, "{}", logs)?;
        }
        Ok(())
    }

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

    pub fn spawn_service_log_writer(&self, service: &str) -> Result<()> {
        let service = service.to_string();
        thread::spawn(move || {
            let mut last_fetch_time = SystemTime::now();
            loop {
                let since = format!("{:?}", last_fetch_time);
                match LogWriter::extract_service_logs(&service, &since) {
                    Ok(logs) => {
                        info!("{}", logs);
                        last_fetch_time = SystemTime::now();
                    }
                    Err(e) => eprintln!("Failed to fetch logs: {} {}", e, service),
                }
                thread::sleep(Duration::from_secs(1));
            }
        });

        Ok(())
    }

    pub fn spawn_kernel_log_writer(&self) -> Result<()> {
        thread::spawn(move || {
            let mut last_fetch_time = SystemTime::now();
            loop {
                let since = format!("{:?}", last_fetch_time);
                match LogWriter::extract_kernel_logs(&since) {
                    Ok(logs) => {
                        info!("{}", logs);
                        last_fetch_time = SystemTime::now();
                    }
                    Err(e) => eprintln!("Failed to fetch logs: {}", e),
                }
                thread::sleep(Duration::from_secs(1));
            }
        });

        Ok(())
    }
}
