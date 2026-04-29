# Project State

## Current Focus
Improve async input handling for `dracon-terminal-engine` by adding support for non-blocking polling and implementation of an async reader.

## Completed
- feat(input): Implement non-blocking poll `poll_input_async` (used in the TTY backend), with polling timeout handling and event reporting
- feat(input): Implement asynchronous reader entry point `spawn`
- feat(input): Implement variant of asynchronous reader using a channel (`spawn_with_channel`), useful in distributed systems.
- feat(input): Implement blocking mode variant of async reader with Tokio runtime spawning and blocking execution
- chore(docs): Reflect asynchronous capabilities in the README for terminal-engine users.
- dead_code: Cleanup compose plane module by removing unnecessary `[allow(dead_code)]` annotations.

## In Progress
- integrate newly implemented `async` input handling into application workflows and UIs

## Next Steps
- WIP: Refactor and test asynchronous reading implementation for robustness and performance.
