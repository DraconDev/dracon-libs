# Project State

## Current Focus
Remove internal cell storage and sorting configuration from Table to simplify rendering.

## Completed
- [x] Removed `cells: Vec<String>` field from `Column` struct
- [x] Removed `sort_col`, `sort_asc` fields from `Table` struct
- [x] Removed `on_sort` callback from `Table` struct
