# Project State

## Current Focus
Expose command execution and registry via the app context while removing Label widget and tightening status badge dependencies.

## Completed
- [x] Add synchronous command runner and available-commands getter on Ctx so callers can invoke and inspect registered commands.
- [x] Remove Label widget from public exports to streamline the widget module surface.
- [x] Import Widget trait in status_badge.rs to prepare for trait-based rendering or configuration without affecting runtime behavior.
