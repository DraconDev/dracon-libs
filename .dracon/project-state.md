# Project State

## Current Focus
Normalize commit summary prefixes, simplify widget index calculations, and ignore TTY‑dependent smoke test.

## Completed
- [x] Made `summary` mutable in `extract_focus_summary` and strip optional "ONE LINE:"/"one line:" prefixes before truncation
- [x] Replaced complex index calculation `(0u16 * plane.width + i as u16) as usize` with plain `i` in `StatusBar` rendering
- [x] Replaced complex index calculation `(0u16 * plane.width + i as u16) as usize` with plain `i` in `Toast` rendering
- [x] Added `#[ignore = "requires a real TTY; stdout is piped in CI/test environments so this hangs"]` attribute to `test_text_editor_demo_smoke` to prevent hanging in CI
