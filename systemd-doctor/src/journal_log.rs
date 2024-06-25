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
    pub fn new(log_file: &str) -> Self {
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
        let log_file_path = self.log_file_path.clone();
        thread::spawn(move || {
            let log_writer = LogWriter::new(&log_file_path);
            let mut last_fetch_time = SystemTime::now();
            loop {
                let since = format!("{:?}", last_fetch_time);
                match LogWriter::extract_service_logs(&service, &since) {
                    Ok(logs) => {
                        if let Err(e) = log_writer.write_log(&logs) {
                            eprintln!("Failed to write logs to file: {}", e);
                        }
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
        let log_file_path = self.log_file_path.clone();
        thread::spawn(move || {
            let log_writer = LogWriter::new(&log_file_path);
            let mut last_fetch_time = SystemTime::now();
            loop {
                let since = format!("{:?}", last_fetch_time);
                match LogWriter::extract_kernel_logs(&since) {
                    Ok(logs) => {
                        if let Err(e) = log_writer.write_log(&logs) {
                            eprintln!("Failed to write logs to file: {}", e);
                        }
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
