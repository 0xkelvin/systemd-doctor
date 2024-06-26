use std::thread;
use std::time::Duration;

mod cmd_health_check;
mod health_monitor;
mod journal_log;
mod sys_health_check;

fn main() {
    println!("Hello, world!");
    loop {
        let mut health = sys_health_check::HealthCheck::new();
        let mut cmd_health = cmd_health_check::cmdHealCheck::new();
        let log = journal_log::LogWriter::new("service_test_2.log", "service_test_2");
        let service_name = "service_test_2";
        let cpu_load = cmd_health.cmd_check_cpu_load(service_name, None);
        let memory_usage = cmd_health.cmd_check_memory_usage_kb(service_name, None);
        let _ = log.spawn_service_log_writer("service_test_2");
        // let _start_journal = journal_log::spawn_log_writer(service_name, "service_test_2.log");

        // let cpu_load = health.check_cpu_load(service_name, None);
        // let memory_usage = health.check_memory_usage(service_name, None);
        // println!("CPU load: {:?}", cpu_load);
        // println!("Memory usage: {:?}", memory_usage);
        thread::sleep(Duration::from_secs(1));
    }
}
