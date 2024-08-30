use std::fs;
use std::io;
use std::str::FromStr;
use std::{fs::File, io::BufRead};
use std::{num::ParseIntError, process::Command};

pub struct MemInfo {
    pub total_memory: u64,
    pub free_memory: u64,
    pub available_memory: u64,
    pub buffers_memory: u64,
    pub cached_memory: u64,
}

pub struct CmdHealCheck;

impl CmdHealCheck {
    pub fn new() -> Self {
        Self {}
    }

    fn parse_meminfo_value(line: &str) -> u64 {
        let value_in_kb = line
            .split_whitespace()
            .nth(1)
            .unwrap_or("0")
            .parse::<u64>()
            .unwrap_or(0);

        value_in_kb / 1024
    }

    pub fn parse_meminfo(&self) -> io::Result<MemInfo> {
        let file = File::open("/proc/meminfo")?;
        let reader = io::BufReader::new(file);

        let mut total_memory = 0;
        let mut free_memory = 0;
        let mut available_memory = 0;
        let mut buffers_memory = 0;
        let mut cached_memory = 0;

        for line in reader.lines() {
            let line = line?;

            if line.starts_with("MemTotal:") {
                total_memory = Self::parse_meminfo_value(&line);
            } else if line.starts_with("MemFree:") {
                free_memory = Self::parse_meminfo_value(&line);
            } else if line.starts_with("MemAvailable:") {
                available_memory = Self::parse_meminfo_value(&line);
            } else if line.starts_with("Buffers:") {
                buffers_memory = Self::parse_meminfo_value(&line);
            } else if line.starts_with("Cached:") {
                cached_memory = Self::parse_meminfo_value(&line);
            }
        }

        Ok(MemInfo {
            total_memory,
            free_memory,
            available_memory,
            buffers_memory,
            cached_memory,
        })
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

    pub fn cmd_check_memory_usage_mb(&self, service: &str) -> Result<u64, String> {
        match self.cmd_check_memory_usage_kb(service, None) {
            Ok(total_mem_kb) => Ok(total_mem_kb / 1024),
            Err(e) => Err(format!("Failed to read memory usage of service: {}", e)),
        }
    }

    pub fn cmd_check_memory_usage_kb(
        &self,
        service: &str,
        _threshold_kb: Option<u64>,
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
        // println!("{}: memory: {}", service, total_memory_kb);

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

    pub fn get_cpu_temperature(&self) -> Result<f64, String> {
        let path = "/sys/class/thermal/thermal_zone0/temp";
        // let contents = fs::read_to_string(path)?;
        match fs::read_to_string(path) {
            Ok(contents) => {
                let temp_milidegress: f64 = contents
                    .trim()
                    .parse()
                    .map_err(|e| format!("Failed to parse temperature file: {}", e))?;
                Ok(temp_milidegress / 1000.0)
            }
            Err(e) => Err(format!("Failed to read temperature file: {}", e)),
        }
    }
}
