use crate::LogWriter;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};



pub struct HealthMonitor {
    services: Vec<String>,
    check_interval: Duration, 
    log_writer: LogWriter,
}

impl HealthMonitor {
    pub fn new(service: Vec<String>, check_interval: Duration, log_file: &str, config_file: &str) -> Self {
        let log_writer = LogWriter::new(log_file, config_file);

        Self {
            services: service,
            check_interval,
            log_writer,
        }
    }

    pub fn start(&self) {
        for service in &self.service {
            let service_name = service.clone();
            let log_writer = self.log_writer.clone();
            thread::spawn(move || {
                let mut last_fetch_time = SystemTime::now();
                loop {
                    since = format!("{:?}", last_fetch_time);
                    match LogWriter::extract_service_logs(&service_name, &since) {
                        Ok(logs) => {
                            if let Err(e) = log_writer.write_log(&logs) {
                                eprintln!("Failed to write logs: {}", e);
                            }
                            last_fetch_time = SystemTime::now();
                        }
                        Err(e) => eprintln!("Failed to fetch logs: {} {}", e, service_name),
                    }
                    thread::sleep(self.check_interval);
                }

            });
        }
        // Spawn a thread to monitor the kernel logs
        let kernel_log_writer = self.log_writer.clone();
        thread::spawn(|| {
            let mut last_fetch_time = SystemTiime::now();
            loop {
                since = format!{"{:?}", last_fetch_time};
                match LogWriter::extract_kernel_logs(&since){
                    Ok(logs) => {
                        if let Err(e) = kernel_log_writer.write_log(&logs) {
                            eprintln!("Failed to write logs: {}", e);
                        }
                    }
                    Err(e) => eprintln!("Failed to fetch logs: {}", e),
                }
                thread::sleep(self.check_interval);
            }
        });
    }
}
