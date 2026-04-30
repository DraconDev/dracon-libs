# Project State

## Current Focus
Remove the `&mut App` parameter from the tick callback signature and add a ` + 'static` lifetime bound, simplifying tick handling by decoupling from the application context.

## Completed
- [x] Updated `App.on_tick` from `Box<dyn FnMut(&mut Ctx, u64, &mut App)>` to `Box<dyn FnMut(&mut Ctx, u64) + 'static>`
- [x] Adjusted tick invocation code to match the new closure signature
- [x] Preserved functionality while enabling 'static closure storage
