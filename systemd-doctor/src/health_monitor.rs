use chrono::Local;

use crate::cmd_health_check::CmdHealCheck;
use crate::log::LogWriter;
use std::fs::metadata;
use std::io;
use std::time::Duration;

pub struct HealthMonitor {
    services: Option<Vec<String>>,
    check_interval: Duration,
    log_writer: LogWriter,
    cmd_checker: CmdHealCheck,
}

impl HealthMonitor {
    pub fn new(
        services: Option<Vec<String>>,
        check_interval: Duration,
        log_file: Option<&str>,
    ) -> Result<Self, io::Error> {
        let log_writer = LogWriter::new(log_file)?;
        let cmd_checker = CmdHealCheck::new();

        Ok(Self {
            services,
            check_interval,
            log_writer,
            cmd_checker,
        })
    }

    pub fn start_monitor_memory(&mut self) -> Result<(), io::Error> {
        if metadata(self.log_writer.get_log_file_path())?.len() == 0 {
            let header = [
                "Timestamp",
                "Total Memory (MB)",
                "Free Memory (MB)",
                "Available Memory (MB)",
                "Buffers Memory (MB)",
                "Cached Memory (MB)",
            ];
            self.log_writer.write_record(&header)?;
        }

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        match self.cmd_checker.parse_meminfo() {
            Ok(meminfo) => {
                let record = [
                    timestamp.as_str(),
                    &meminfo.total_memory.to_string(),
                    &meminfo.free_memory.to_string(),
                    &meminfo.available_memory.to_string(),
                    &meminfo.buffers_memory.to_string(),
                    &meminfo.cached_memory.to_string(),
                ];
                self.log_writer.write_record(&record)?;
            }
            Err(e) => {
                eprintln!("Failed to retrieve memory information: {}", e);
            }
        }

        Ok(())
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
