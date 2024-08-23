use csv::Writer;
use log::info;
use std::env;
use std::fs::OpenOptions;
use std::io;
use std::io::Result;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::{Duration, SystemTime};

pub struct LogWriter {
    log_file_path: PathBuf,
    writer: Writer<std::fs::File>,
}

impl LogWriter {
    pub fn new(log_file: Option<&str>) -> io::Result<Self> {
        let log_file_path = match log_file {
            Some(path) => PathBuf::from(path),
            None => {
                let mut current_dir = env::current_dir().expect("Failed to get currect directory");
                current_dir.push("DrViet_health_log.csv");
                current_dir
            }
        };

        // Open or create the csv file
        let file = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(&log_file_path)?;
        let writer = Writer::from_writer(file);

        Ok(Self {
            log_file_path,
            writer,
        })
    }

    pub fn get_log_file_path(&self) -> &PathBuf {
        &self.log_file_path
    }

    //methods to write to the csv file
    pub fn write_record(&mut self, record: &[&str]) -> io::Result<()> {
        self.writer.write_record(record)?;
        let _ = self.writer.flush(); // ensure data is written to the file
        Ok(())
    }

    pub fn log_info(&self, message: &str) {
        info!("{}", message);
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
