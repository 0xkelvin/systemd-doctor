use crate::cmd_health_check::cmdHealCheck;
use crate::journal_log::LogWriter;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct HealthMonitor {
    services: Vec<String>,
    check_interval: Duration,
    log_writer: LogWriter,
    cmd_checker: cmdHealCheck,
}

impl HealthMonitor {
    pub fn new(
        service: Vec<String>,
        check_interval: Duration,
        log_file: &str,
        config_file: &str,
    ) -> Self {
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
        for service in &self.services {
            let service_name = service.clone();
            let log_writer = self.log_writer.clone();
            let cmd_checker = self.cmd_checker.clone();
            let check_interval = self.check_interval;
            thread::spawn(move || loop {
                let cpu_load = cmd_checker
                    .cmd_check_cpu_load(&service_name, None)
                    .unwrap_or(0.0);
                let memory_usage = cmd_checker
                    .cmd_check_memory_usage_kb(&service_name, None)
                    .unwrap_or(0);
                let cpu_temp = cmd_checker.get_cpu_temperature().unwrap_or(0.0);
                let log_message = format!(
                    "{}: CPU Load: {}%, Memory Usage: {} KB, CPU Team: {:.2}C",
                    service_name, cpu_load, memory_usage, cpu_temp
                );

                log_writer.log_info(&log_message);
                thread::sleep(check_interval);
            });
        }
    }

    pub fn enable_journal_service_log(&self, service_name: &str) -> Result<(), String> {
        self.log_writer
            .spawn_service_log_writer(service_name)
            .map_err(|e| format!("Failed to start service log: {}", e))
    }

    pub fn enable_journal_kernel_log(&self) -> Result<(), String> {
        self.log_writer
            .spawn_kernel_log_writer()
            .map_err(|e| format!("Failed to start kernel log: {}", e))
    }
}
