# ProjectState

## Current Focus
Refactor README to present dracon-terminal-engine as a full terminal framework with new examples and detailed component documentation.

## Completed
- [x] Updated title and description to describe dracon-terminal-engine as a complete terminal application framework.
- [x] Added “One import to rule them all” section with a concise `App::new()` example.
- [x] Removed version tag `tag = "v19.2.2"` from the Cargo.toml dependencies snippet.
- [x] Added detailed “Core” section describing Z‑Indexed Compositor layers and usage examples.
- [x] Added “Input” section covering SGR Mouse, chord support, and input event contract types.
- [x] Added “Visuals” section describing truecolor, visual filters, and synchronized rendering.
- [x] Added “Editor Widget” section covering syntax highlighting, smart filters, unlimited undo/redo, and multi‑selection.
- [x] Introduced “Framework (v25)” heading with comprehensive subsections (Core, Input, Visuals, Editor Widget).
- [x] Added a widget reference table listing App, Ctx, List<T>, Table<T>, TabBar, Breadcrumbs, SplitPane, Modal, ContextMenu, HUD, HitZone<T>, HitZoneGroup<T>, ScrollContainer, Theme.
- [x] Added “Engine (Core)” section with Ratatui bridge description and a module reference table (compositor, input parser, reader, widgets, integration, backend, visuals, system).
- [x] Added “Quick Start (Framework)” example code demonstrating a minimal application using the framework.
- [x] Added “Quick Start (Engine-level)” example code showing low‑level compositor and input handling.
- [x] Updated dependencies snippet to drop the version tag while keeping the git source.
