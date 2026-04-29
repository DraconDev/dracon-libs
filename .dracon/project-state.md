# Project State

## Current Focus
Fix bounds handling in the compositor and improve parsing of system disk information.

## Completed
- [x] refactor(compositor): replace direct addition with `saturating_add` to prevent overflow when computing plane boundaries and pixel coordinates, ensuring safe bounds checks during rendering.
- [x] refactor(system_monitor): change disk info parsing to split on null characters (`'\x00'`), filter out empty segments, enforce a minimum number of fields, and safely unwrap optional values, eliminating crashes on malformed output.
