use std::fs;
use std::str::FromStr;
use std::{num::ParseIntError, process::Command};

pub struct cmdHealCheck;

impl cmdHealCheck {
    pub fn new() -> Self {
        Self {}
    }

    /*
    sh -c "ps -C <service name> -o %cpu= | awk '{s+=\$1} END {print s}'"
    */
    // not implement the threshold yet
    pub fn cmd_check_cpu_load(
        &self,
        service: &str,
        _threshold: Option<f32>,
    ) -> Result<f32, String> {
        let output = Command::new("sh")
            .arg("-c")
            .arg(format!(
                "ps -C {} -o %cpu= | awk '{{s+=$1}} END {{print s}}'",
                service
            ))
            .output()
            .expect("Failed to execute command");

        let load: f32 = String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse()
            .unwrap_or(0.0);
        println!("{}: cpu_load: {}", service, load);
        Ok(load)
    }

    pub fn cmd_check_memory_usage_kb(
        &self,
        service: &str,
        _threshold_kb: u64,
    ) -> Result<u64, String> {
        let mut total_memory_kb: u64 = 0;

        let output = Command::new("pgrep")
            .arg(service)
            .output()
            .map_err(|e| format!("Failed to execute command: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to execute command: {}", stderr));
        }

        let pids = String::from_utf8_lossy(&output.stdout)
            .trim()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        if pids.is_empty() {
            return Err(format!("Service {} not found", service));
        }

        for pid in pids {
            let statm_path = format!("/proc/{}/statm", pid);
            let statm = fs::read_to_string(statm_path)
                .map_err(|e| format!("Failed to read file: {}", e))?;

            let mem_pages: u64 = statm
                .split_whitespace()
                .next()
                .ok_or_else(|| format!("Failed to parse statm file for PID {}", pid))?
                .parse()
                .map_err(|e| format!("Failed to parse memory usage for PID {}: {}", pid, e))?;

            total_memory_kb += mem_pages * 4; // 4 KB per page
        }
        println!("{}: memory: {}", service, total_memory_kb);

        Ok(total_memory_kb)
    }

    pub fn cmd_get_total_used_and_free_disk_space(&self) -> Result<(u64, u64, u64), String> {
        let output = Command::new("sh")
            .arg("-c")
            .arg("df / --output=size,avail | tail -n1")
            .output()
            .map_err(|e| format!("Failed to execute command: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to execute command: {}", stderr));
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = stdout.trim().split_whitespace().collect();
        if parts.len() != 3 {
            return Err(format!("Failed to parse command output: {}", stdout));
        }

        let total_space =
            u64::from_str(parts[0]).map_err(|e| format!("Failed to parse total space: {}", e))?;
        let used_space =
            u64::from_str(parts[1]).map_err(|e| format!("Failed to parse free space: {}", e))?;
        let free_space = total_space - used_space;
        Ok((total_space, used_space, free_space))
    }
}
