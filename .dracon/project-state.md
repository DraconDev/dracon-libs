# Project State

## Current Focus
refactor(tabbar): simplify TabBar widget by removing lifetime parameters and width configuration

## Completed
- [x] Remove lifetime parameter from TabBar struct, converting tabs from Vec<&str> to Vec<String>
- [x] Simplify constructor to accept Vec<&str> and internally convert to owned Strings
- [x] Remove width field and with_width method, using hardcoded 80 for mouse handling and area.width for rendering
- [x] Simplify render and handle_mouse methods by consolidating tab count calculations
- [x] Fix HitZone return type from HitZone<'static, usize> to HitZone<usize>
