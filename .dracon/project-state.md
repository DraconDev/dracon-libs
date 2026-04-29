# Project State

## Current Focus
Expose compositor and input parser APIs and modernize module documentation for downstream use.

## Completed
- [x] Re-export `Compositor`, `Plane`, `Cell`, `Color`, `Styles` from compositor for external consumers.
- [x] Re-export `Parser` from input to enable external parsing of SGR mouse and chord sequences.
- [x] Replace stale module comments with `#[doc = "..."]` attributes for consistent documentation generation.
