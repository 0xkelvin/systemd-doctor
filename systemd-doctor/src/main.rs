use std::io;
use std::thread;
use std::time::Duration;
mod cmd_health_check;
mod config;
mod health_monitor;
mod log;
mod sys_health_check;
use crate::health_monitor::HealthMonitor;
use std::sync::{Arc, Mutex};

fn main() -> io::Result<()> {
    println!("Starting health check...");

    let health_monitor = Arc::new(Mutex::new(
        HealthMonitor::new("config.toml", Duration::from_secs(10), None).unwrap(),
    ));

    HealthMonitor::start_tracking(health_monitor.clone(), Duration::from_secs(10));

    loop {
        println!("Viet is working");
        thread::sleep(Duration::from_secs(10));
    }
}
