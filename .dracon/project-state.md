# Project State

## Current Focus
This commit introduces several structural updates to the focus management utilities within the Dracon Framework, aiming to improve widget interaction patterns and internal state handling.

## Completed Changes
- Updated callback types to better support focus events, including new types for trap and focus change notifications.
- Modified `FocusManager` to integrate tracking of focus events via updated struct members and methods.
- Enhanced tracking of focus tab order and event internal states to avoid conflicts with existing UI logic.
- Refactored `FocusManager` implementation to manage callback registration and desregistration more clearly.
- Added logic for tracking whether focus traps are active, enabling better UI and UI management.
