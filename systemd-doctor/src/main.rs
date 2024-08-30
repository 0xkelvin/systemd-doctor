use std::io;
use std::thread;
use std::time::Duration;
mod cmd_health_check;
mod config;
mod health_monitor;
mod log;
mod sys_health_check;
use crate::health_monitor::HealthMonitor;

fn main() -> io::Result<()> {
    println!("Starting health check...");

    let mut health_track = HealthMonitor::new("./config.toml", Duration::new(5, 0), None)
        .expect("Failed to create heal monitoring");

    loop {
        let _ = health_track.start_monitor_memory();
        thread::sleep(Duration::from_secs(10));
    }
}
