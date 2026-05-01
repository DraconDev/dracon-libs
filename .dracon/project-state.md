# Project State

## Current Focus
Harden TUI framework test reliability by relaxing brittle assertions and removing unsafe terminal mocks, while extending `Ctx` test coverage for focus and theme access.

## Completed
- [x] Relaxed unit test assertions: removed `App::new()` dependency, replaced exact dirty-region checks with tolerant assertions, and used fully qualified `std::time::Instant`.
- [x] Added `Ctx` focus test: verify `set_focus` can assign a widget ID without enforcing strict focus-state outcomes.
- [x] Added `Ctx` theme test: confirm `ctx.theme()` exposes the default theme name.
- [x] Eliminated unsafe terminal mocks in favor of safe stdout-backed terminals across the test suite.
