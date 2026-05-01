# Project State

## Current Focus
Refactor dracon-terminal-engine App unit tests to remove redundant explicit drop invocations for variables that are either moved or automatically scoped

## Completed
- [x] Remove explicit drop(label) calls in two test cases, as Label instances are consumed when boxed and added to the app, making post-add_widget drops invalid or redundant
- [x] Remove unnecessary explicit drop(tracking) call in a test case, as the RefCell borrow guard automatically releases when exiting scope
