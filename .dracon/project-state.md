# Project State

## Current Focus
Clean up redundant AI contract code by removing unused parsing logic and duplicate struct definitions.

## Completed
- [x] Remove unused `RoutingTask::parse` method and its match implementation.
- [x] Delete duplicate `SelectionConstraints` struct definition that conflicted with the existing one.
- [x] Simplify `dracon-ai-contracts` library by keeping a single, canonical `SelectionConstraints` definition.
