use chrono::Local;

use crate::cmd_health_check::CmdHealCheck;
use crate::config::ConfigParser;
use crate::log::LogWriter;
use std::fmt::format;
use std::fs::metadata;
use std::io;
use std::time::Duration;

pub struct HealthMonitor {
    services: Option<Vec<String>>,
    check_interval: Duration,
    memory_log: LogWriter,
    cpu_log: LogWriter,
    cmd_checker: CmdHealCheck,
}

impl HealthMonitor {
    pub fn new(
        config_path: &str,
        check_interval: Duration,
        _log_file: Option<&str>,
    ) -> Result<Self, io::Error> {
        let (memory_log, cpu_log) = LogWriter::new()?;
        let cmd_checker = CmdHealCheck::new();
        let config = ConfigParser::new(config_path);
        let tracking_services = config.get_config_services().clone();
        Ok(Self {
            services: tracking_services,
            check_interval,
            memory_log,
            cpu_log,
            cmd_checker,
        })
    }

    pub fn start_monitor_memory(&mut self) -> Result<(), io::Error> {
        if metadata(self.memory_log.get_log_file_path())?.len() == 0 {
            let mut header = vec![
                "Timestamp".to_string(),
                "Total Memory(MB)".to_string(),
                "Free Memory(MB)".to_string(),
                "Available Memory(MB)".to_string(),
                "Buffers Memory(MB)".to_string(),
                "Cached Memory(MB)".to_string(),
            ];

            // Extend header with service names
            if let Some(services) = &self.services {
                for service in services {
                    header.push(format!("{}(MB)", service));
                }
            }

            let header_refs: Vec<&str> = header.iter().map(String::as_str).collect();
            self.memory_log.write_record(&header_refs)?;
        }

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        match self.cmd_checker.parse_meminfo() {
            Ok(meminfo) => {
                let mut record = vec![
                    timestamp,
                    meminfo.total_memory.to_string(),
                    meminfo.free_memory.to_string(),
                    meminfo.available_memory.to_string(),
                    meminfo.buffers_memory.to_string(),
                    meminfo.cached_memory.to_string(),
                ];

                if let Some(services) = &self.services {
                    for service in services {
                        match self.cmd_checker.cmd_check_memory_usage_mb(service) {
                            Ok(memory_usage) => record.push(memory_usage.to_string()),
                            Err(e) => {
                                eprintln!("Failed to get memory usage for {}: {}", service, e);
                                record.push("N/A".to_string()); // Or use a default value
                            }
                        }
                    }
                }
                let record_refs: Vec<&str> = record.iter().map(String::as_str).collect();
                self.memory_log.write_record(&record_refs)?;
            }
            Err(e) => {
                eprintln!("Failed to retrieve memory information: {}", e);
            }
        }

        Ok(())
    }

    pub fn start_monitor_cpuload(&mut self) -> Result<(), io::Error> {
        if metadata(self.cpu_log.get_log_file_path())?.len() == 0 {
            let mut header = vec!["Timestamp".to_string()];

            if let Some(services) = &self.services {
                for service in services {
                    header.push(format!("{}(MB)", service));
                }
            }

            let header_refs: Vec<&str> = header.iter().map(String::as_str).collect();
            self.cpu_log.write_record(&header_refs)?;
        }

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let mut record = vec![timestamp];
        if let Some(services) = &self.services {
            for service in services {
                match self.cmd_checker.cmd_check_cpu_load(service, None) {
                    Ok(cpu_load) => record.push(cpu_load.to_string()),
                    Err(e) => {
                        eprintln!("Failed to get cpu usage for {}: {}", service, e);
                        record.push("N/A".to_string()); // Or use a default value
                    }
                }
            }

            let record_refs: Vec<&str> = record.iter().map(String::as_str).collect();
            self.cpu_log.write_record(&record_refs)?;
        }

        Ok(())
    }

    // pub fn enable_journal_service_log(&self, service_name: &str) -> Result<(), String> {
    //     self.log_writer
    //         .spawn_service_log_writer(service_name)
    //         .map_err(|e| format!("Failed to start service log: {}", e))
    // }

    // pub fn enable_journal_kernel_log(&self) -> Result<(), String> {
    //     self.log_writer
    //         .spawn_kernel_log_writer()
    //         .map_err(|e| format!("Failed to start kernel log: {}", e))
    // }
}
