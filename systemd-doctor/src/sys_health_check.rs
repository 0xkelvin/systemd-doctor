use std::{os::unix::process, thread, time::Duration};
use sysinfo::System;

pub struct HealthCheck {
    system: System,
}

impl HealthCheck {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        Self { system }
    }

    pub fn check_cpu_load(
        &mut self,
        service_name: &str,
        _threshold: Option<f32>,
    ) -> Result<f32, String> {
        self.system.refresh_all();
        thread::sleep(Duration::from_millis(500));
        self.system.refresh_all();
        for process in self.system.processes_by_exact_name(service_name) {
            println!("{}: cpu_load: {}", process.name(), process.cpu_usage());
            return Ok(process.cpu_usage());
        }
        Err(format!("Service {} not found or CPU usage", service_name))
    }

    pub fn check_memory_usage(
        &mut self,
        service_name: &str,
        _threshold: Option<u64>,
    ) -> Result<u64, String> {
        self.system.refresh_all();
        thread::sleep(Duration::from_millis(500));
        self.system.refresh_all();
        for process in self.system.processes_by_exact_name(service_name) {
            println!("{}: memory: {}", process.name(), process.memory());
            return Ok(process.memory());
        }
        Err(format!("Service {} not found or CPU usage", service_name))
    }
}

#[cfg(test)]
mod tests {
    use crate::sys_health_check::HealthCheck;

    #[test]
    fn test_check_cpu_load_on_valid_service() {
        let mut health_check = HealthCheck::new();
        let service_name = "bash";
        let cpu_load = health_check.check_cpu_load(service_name, None);
        assert!(cpu_load.is_ok());
    }

    #[test]
    fn test_check_cpu_load_on_invalid_service() {
        let mut health_check = HealthCheck::new();
        let service_name = "invalid_service";
        let cpu_load = health_check.check_cpu_load(service_name, None);
        assert!(cpu_load.is_err());
    }
}
