# Project State

## Current Focus
feat(toast): expose `message()` method and adjust example usage

## Completed
- [x] Added `pub fn message(&self) -> &str` to `Toast` returning its stored message field
- [x] Modified example to call `toast.message()` instead of the removed `toast.text()`
- [x] Updated debug output to print render dimensions of the Checkbox widget
- [x] Updated `Cargo.lock` with newer dependency versions
