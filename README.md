![image](https://github.com/user-attachments/assets/a33429e9-7961-419f-92c9-511f383cb273)




# Overview
_**Systemd-doctor**_ is a health monitoring service designed to track and manage the health of various services on an embedded Linux device. 

It integrates with _**Systemd**_ to automatically restart services when abnormalities are detected, making sure your custom services are working. 

Additionally, _**Systemd-doctor**_ stores metrics in a _time-series database_, allowing users to view metrics and charts. It is helpful for System Analysis when we need a comprehensive data to evaluate out custom services and resouce, good information for debugging too. 

_**Systemd-doctor**_ service is able to reset itself by _**Systemd Watchdog**_

# Features
- Monitors CPU load, memory usage, disk space, and service status of "services"...
- Tracks global metrics like CPU temperature, board temperature, and network bandwidth...
- Journal-logging for each of service and kernel log
- Automatically restarts services if thresholds are breached.
- Validates if the services specified for tracking are valid systemd services.
- Stores metrics in a time-series database for visualization in Grafana.

# Configuration 
### Tracking Services Registration 
The configuration file (config.toml) allows users to specify the services to monitor and their respective thresholds.
Example 
```
[services]
list = ["ota", "mqtt-client", "can-parser", "logging"]

[thresholds.ota]
cpu = 80.0
memory = 70.0
disk = 90

[thresholds.mqtt-client]
cpu = 60.0
memory = 50.0
disk = 85

[thresholds.can-parser]
cpu = 75.0
memory = 65.0
disk = 88

[thresholds.logging]
cpu = 70.0
memory = 60.0
disk = 85

[global_thresholds]
cpu_temperature = 80.0
board_temperature = 70.0
network_bandwidth = 1000.0 

```
### Service file for Systemd-doctor
```
[Unit]
Description=Doctor Viet - Health Monitoring Service
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/systemd-doctor --config=/path/to/config.toml
WatchdogSec=10
Restart=always

[Install]
WantedBy=multi-user.target
```
