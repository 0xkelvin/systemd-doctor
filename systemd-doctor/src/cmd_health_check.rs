use std::fmt::format;
use std::fs;
use std::io;
use std::io::stderr;
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;
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
        let pids = self.get_service_pids(service)?;

        if pids.is_empty() {
            return Err(format!("Service {} not found", service));
        }

        let mut total_cpu_usage = 0.0;

        // First snapshot
        let mut first_stat = vec![];
        for pid in &pids {
            first_stat.push(self.read_stat(pid)?);
        }
        let first_uptime = self.read_uptime()?;

        // Sleep for a short duration to calculate the CPU load over time
        sleep(Duration::from_secs(1));

        // Second snapshot
        let mut second_stat = vec![];
        for pid in &pids {
            second_stat.push(self.read_stat(pid)?);
        }
        let second_uptime = self.read_uptime()?;

        for i in 0..pids.len() {
            let cpu_usage = self.calculate_cpu_usage(
                &first_stat[i],
                &second_stat[i],
                first_uptime,
                second_uptime,
            );
            total_cpu_usage += cpu_usage;
        }

        // Truncate the CPU load to one decimal place
        let truncated_load = (total_cpu_usage * 10.0).trunc() / 10.0;

        Ok(truncated_load)
    }

    fn get_service_pids(&self, service: &str) -> Result<Vec<u32>, String> {
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
            .map(|s| s.parse::<u32>().unwrap_or(0))
            .collect::<Vec<u32>>();

        Ok(pids)
    }

    fn read_stat(&self, pid: &u32) -> Result<(u64, u64), String> {
        let stat_path = format!("/proc/{}/stat", pid);
        let stat = fs::read_to_string(stat_path)
            .map_err(|e| format!("Failed to read stat file for PID {}: {}", pid, e))?;
        let stat_values: Vec<&str> = stat.split_whitespace().collect();

        let utime: u64 = stat_values[13]
            .parse()
            .map_err(|e| format!("Failed to parse utime: {}", e))?;
        let stime: u64 = stat_values[14]
            .parse()
            .map_err(|e| format!("Failed to parse stime: {}", e))?;

        Ok((utime, stime))
    }

    fn read_uptime(&self) -> Result<f64, String> {
        let uptime_str = fs::read_to_string("/proc/uptime")
            .map_err(|e| format!("Failed to read uptime: {}", e))?;
        let uptime: f64 = uptime_str
            .split_whitespace()
            .next()
            .unwrap_or("0")
            .parse()
            .map_err(|e| format!("Failed to parse uptime: {}", e))?;

        Ok(uptime)
    }

    fn calculate_cpu_usage(
        &self,
        first_stat: &(u64, u64),
        second_stat: &(u64, u64),
        first_uptime: f64,
        second_uptime: f64,
    ) -> f32 {
        let total_time_first = first_stat.0 + first_stat.1;
        let total_time_second = second_stat.0 + second_stat.1;
        let total_time_diff = total_time_second as f64 - total_time_first as f64;

        let elapsed_time = second_uptime - first_uptime;

        if elapsed_time > 0.0 {
            (total_time_diff / (elapsed_time * 100.0)) as f32
        } else {
            0.0
        }
    }

    // using the VmRSS field from the /proc/[pid]/status file
    pub fn cmd_check_memory_usage_mb(
        &self,
        service: &str,
        _threshold_mb: Option<u64>,
    ) -> Result<f64, String> {
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
            let status_path = format!("/proc/{}/status", pid);
            let status = fs::read_to_string(&status_path)
                .map_err(|e| format!("Failed to read file: {}", e))?;
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    let rss_kb: u64 = line
                        .split_whitespace()
                        .nth(1)
                        .ok_or_else(|| format!("Failed to parse VmRSS for PID {}", pid))?
                        .parse()
                        .map_err(|e| {
                            format!("Failed to parse memory usage for PID {}: {}", pid, e)
                        })?;
                    total_memory_kb += rss_kb;
                }
            }
        }

        // Convert the total memory from KB to MB and truncate to 1 decimal place
        let total_memory_mb = (total_memory_kb as f64 / 1024.0 * 10.0).trunc() / 10.0;

        Ok(total_memory_mb)
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
