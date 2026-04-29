# Project State

## Current Focus
Add parser initialization and timeout handling, and implement system monitoring data structures and refresh logic

## Completed
- [x] Add `Parser::new` method that creates a parser with an empty buffer of capacity 32
- [x] Add `Parser::check_timeout` method that emits an `Esc` key event when an incomplete escape sequence times out
- [x] Define `DiskInfo` struct with disk name, device path, used/available/total space, and mount status
- [x] Define `ProcessInfo` struct with PID, name, CPU usage, memory usage, owner, and status
- [x] Define `SystemData` struct aggregating CPU, memory, disk, process, network, uptime, OS, kernel, and hostname data
- [x] Implement `SystemMonitor::new` to create a monitor and refresh all system data on construction
- [x] Implement `SystemMonitor::get_data` to refresh metrics and return a `SystemData` snapshot
- [x] Add `Serialize`/`Deserialize` derives to all data structures for serialization
