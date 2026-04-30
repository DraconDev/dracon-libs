# Project State
This commit introduces changes to the terminal wrapper in Dracon, updating internal state tracking and ensuring safe cleanup. The main focus is on refining the RAII-like behavior when a struct is dropped, with improvements to how terminal attributes and widget dirty states are managed.

## Completed
- Updated trait implementations to properly handle terminal attribute restoration.
- Added necessary boilerplate for widgets to register dirty-state logic.
- Revised file ordering and included necessary imports for file handling.
