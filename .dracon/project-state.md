# Project State

## Current Focus One line

Updated input parsing logic for terminal UI to handle specific key combinations and mouse events more effectively.

## Completed
- [x] Changed Ctrl+. to Ctrl+_ (ASCII 31/Unit Separator) handling, modifying char output from '.' to '_' in key event generation
- [x] Refactored ANSI escape sequence parsing by removing '2' from conditional checks for sequence validation (e.g., function keys), potentially expanding handled cases
- [x] Simplified mouse button detection by reducing bitmask from 0b1100_1011 to 0b0000_0011, focusing detection on lower two bits for standard left/right/middle click handling
