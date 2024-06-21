use std::{num::ParseIntError, process::Command};
use std::fs;
/*
sh -c "ps -C <service name> -o %cpu= | awk '{s+=\$1} END {print s}'"
*/
// not implement the threshold yet
pub fn cmd_check_cpu_load(service: &str, _threshold: Option<f32>) -> Result<f32, String> {
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

pub fn check_memory_usage_kb(service: &str, _threshold_kb: u64) -> Result<u64, String> {
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
        let statm = fs::read_to_string(statm_path).map_err(|e| format!("Failed to read file: {}", e))?;

        let mem_pages: u64 = statm.split_whitespace()
            .next()
            .ok_or_else(|| format!("Failed to parse statm file for PID {}", pid))?
            .parse()
            .map_err(|e| format!("Failed to parse memory usage for PID {}: {}", pid, e))?;

        total_memory_kb += mem_pages * 4; // 4 KB per page

    }
    println!("{}: memory: {}", service, total_memory_kb);

    Ok(total_memory_kb)
}
