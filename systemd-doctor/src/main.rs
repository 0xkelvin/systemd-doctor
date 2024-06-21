use std::thread;
use std::time::Duration;

mod cmd_health_check;
mod sys_health_check;

fn main() {
    println!("Hello, world!");
    loop {
        let mut health = sys_health_check::HealthCheck::new();
        let service_name = "service_test_2";
        let cpu_load = cmd_health_check::cmd_check_cpu_load(service_name, None);
        let memory_usage = cmd_health_check::check_memory_usage_kb(service_name, 0);
        // let cpu_load = health.check_cpu_load(service_name, None);
        // let memory_usage = health.check_memory_usage(service_name, None);
        // println!("CPU load: {:?}", cpu_load);
        // println!("Memory usage: {:?}", memory_usage);
        thread::sleep(Duration::from_secs(1));
    }
}
