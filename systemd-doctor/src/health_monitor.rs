use crate::LogWriter;
use crate::cmdHealCheck;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};



pub struct HealthMonitor {
    services: Vec<String>,
    check_interval: Duration, 
    log_writer: LogWriter,
    cmd_checker: cmdHealCheck,
}

impl HealthMonitor {
    pub fn new(service: Vec<String>, check_interval: Duration, log_file: &str, config_file: &str) -> Self {
        let log_writer = LogWriter::new(log_file, config_file);
        let cmd_checker = cmdHealCheck::new();

        Self {
            services: service,
            check_interval,
            log_writer,
            cmd_checker,
        }
    }

    pub fn start(&self) {
        for service in &self.service {
            let service_name = service.clone();
            let log_writer = self.log_writer.clone();
            let cmd_checker = self.cmd_checker.clone();
            let check_interval = self.check_interval;
            thread::spawn(move || {
                let mut last_fetch_time = SystemTime::now();
                loop {
                    since = format!("{:?}", last_fetch_time);
                    let cpu_load = cmd_checker.cmd_check_cpu_load(&service_name, None).unwrap_or(0.0);
                    let memory_usage = cmd_checker.cmd_check_memory_usage_kb(&service_name, None).unwrap_or(0);
                    let cpu_temp = cmd_checker.get_cpu_temperature().unwrap_or(0.0);
                    let log_message = format!("{}: CPU Load: {}%, Memory Usage: {} KB, CPU Team: {:.2}C",
                        service_name, cpu_load, memory_usage, cpu_temp);

                    if let Err(e) = log_writer.write_log(&log_message) {
                        error!("Failed to write logs for {}: {}", service_name, e);
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
