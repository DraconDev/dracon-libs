# Project State

## Current Focus
Refactor input widgets to share a common text-input base, extracting duplicated logic into text_input_base and simplifying PasswordInput and SearchInput.

## Completed
- [x] Introduce text_input_base module and BaseInput to centralize text buffer, cursor, masking, theming, and submit behavior
- [x] Migrate PasswordInput to delegate to BaseInput, retain mask configuration and password accessor, and reduce per-widget boilerplate
- [x] Migrate SearchInput to delegate to BaseInput, retain search-specific placeholders/behavior, and reduce per-widget boilerplate
- [x] Expose text_input_base in framework/widgets/mod to enable reuse across input widgets
