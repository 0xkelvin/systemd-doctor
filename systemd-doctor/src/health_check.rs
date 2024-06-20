use std::{os::unix::process, thread, time::Duration};

use sysinfo::System;

pub fn check_cpu_load(service_name: &str, _threshold: Option<f32>) -> Result<f32, String> {
    let mut system = System::new_all();
    system.refresh_all();
    thread::sleep(Duration::from_millis(500));
    system.refresh_all();
    for process in system.processes_by_exact_name(service_name) {
        // println!("{}: {}", process.name(), process.cpu_usage());
        return Ok(process.cpu_usage());
    }
    Err(format!("Service {} not found or CPU usage", service_name))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_check_cpu_load_on_valid_service() {
        let service_name = "bash";
        let cpu_load = super::check_cpu_load(service_name, None);
        assert!(cpu_load.is_ok());
    }

    #[test]
    fn test_check_cpu_load_on_invalid_service() {
        let service_name = "invalid_service";
        let cpu_load = super::check_cpu_load(service_name, None);
        assert!(cpu_load.is_err());
    }
}
