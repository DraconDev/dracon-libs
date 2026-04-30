# Project State

## Current Focus
Add multiple predefined color themes and enable split pane divider coloring to adapt to the active theme.

## Completed- [x] Added 12 theme factory methods (dracula, nord, catppuccin_mocha, gruvbox_dark, tokyo_night, solarized_dark, solarized_light, one_dark, rosé_pine, kanagawa, everforest, monokai) to Theme.
- [x] Introduced `divider_color` field in `SplitPane` with default RGB(80,80,100).
- [x] Updated cell styling to use `self.divider_color` instead of a hardcoded RGB value.
- [x] Implemented `on_theme_change` method to sync `divider_color` with `theme.inactive_fg` when the theme changes.
