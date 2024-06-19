use std::process::Command;

/* 
sh -c "ps -C <service name> -o %cpu= | awk '{s+=\$1} END {print s}'"
*/
// not implement the threshold yet
pub fn check_cpu_load(service: &str, _threshold: Option<f32>) -> Result<f32, String> {
    let output = Command::new("sh")
    .arg("-c")
    .arg(format!("ps -C {} -o %cpu= | awk '{{s+=$1}} END {{print s}}'", service))
    .output()
    .expect("Failed to execute command");

let load: f32 = String::from_utf8_lossy(&output.stdout)
    .trim()
    .parse()
    .unwrap_or(0.0);
    Ok(load)
    
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_cpu_load() {
        let service = "service_test_2";
        let cpu_load = check_cpu_load(service, None);
        assert!(cpu_load.is_ok());
        println!("Current CPU load: {:.2}%", cpu_load.unwrap());
    }
}
