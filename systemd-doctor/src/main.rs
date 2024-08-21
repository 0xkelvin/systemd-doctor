use chrono::Local;
use cmd_health_check::cmdHealCheck;
use csv::Writer;
use std::fs::OpenOptions;
use std::io;
use std::thread;
use std::time::Duration;
use std::{
    fs::{metadata, rename},
    io::Write,
};

mod cmd_health_check;
mod health_monitor;
mod journal_log;
mod sys_health_check;

fn main() -> io::Result<()> {
    println!("Starting health check...");

    let health_check = cmdHealCheck::new();
    let file_path = "memory_usage.csv";

    // Open or create the CSV file
    let file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(file_path)?;

    let mut wtr = Writer::from_writer(file);

    // Write the header if the file is empty
    if metadata(file_path)?.len() == 0 {
        wtr.write_record(&[
            "Timestamp",
            "Total Memory (MB)",
            "Free Memory (MB)",
            "Available Memory (MB)",
            "Buffers Memory (MB)",
            "Cached Memory (MB)",
        ])?;
    }

    loop {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        // Fetch memory info and handle any errors
        match health_check.parse_meminfo() {
            Ok(meminfo) => {
                // Write memory information to the CSV file
                wtr.write_record(&[
                    timestamp,
                    meminfo.total_memory.to_string(),
                    meminfo.free_memory.to_string(),
                    meminfo.available_memory.to_string(),
                    meminfo.buffers_memory.to_string(),
                    meminfo.cached_memory.to_string(),
                ])?;
                // Flush the writer to ensure the data is written to the file
                wtr.flush()?;
                println!("Memory information written to CSV.");
            }
            Err(e) => {
                eprintln!("Failed to retrieve memory information: {}", e);
            }
        }

        thread::sleep(Duration::from_secs(10));
    }

    // Unreachable code, but necessary for correct return type
    // Ok(())
}
