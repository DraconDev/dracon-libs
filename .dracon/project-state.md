# Project State

## Current Focus
Add periodic process refresh to limit updates to every 2 seconds, reducing overhead.

## Completed
- [x] added `use std::time::{Duration, Instant};` import
- [x] added `last_process_refresh: Instant` and `process_refresh_interval: Duration` fields to `SystemMonitor`
- [x] initialized those fields in `Default::default()`
- [x] implemented conditional process refresh in `get_data` using elapsed time check
