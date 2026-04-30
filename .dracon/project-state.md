# Project State
##Current Focus
Improved focus management with explicit blur/focus callbacks and Shift‑Tab navigation

## Completed
- [x] Added handling for Shift‑Tab to move focus backward
- [x] Implemented on_blur() notification for the previously focused widget
- [x] Implemented on_focus() notification for the newly focused widget
- [x] Guarded callback invocations to run only when the focus actually changes
- [x] Updated Cargo.lock to reflect new dependency lock (binary change)
