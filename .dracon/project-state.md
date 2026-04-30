# Project State

## Current Focus
Adds widget ID tracking and event limit to EventLogger, and enriches Metric with name, duration, and call count for performance profiling.

## Completed
- [x] Added `id: WidgetId` field to `EventLogger`
- [x] Added `max_events: usize` field to `EventLogger`
- [x] Added `events: VecDeque<LoggedEvent>` field to `EventLogger`
- [x] Added `theme: Theme` field to `EventLogger`
- [x] Added `name: String` field to `Metric`
- [x] Added `value: Duration` field to `Metric`
- [x] Added `call_count: u64` field to `Metric`
