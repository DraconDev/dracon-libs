# Project State

## CurrentFocus
Refactor breadcrumb handling to own string data and adjust the example to provide static string references.

## Completed
- [x] Changed example code to collect breadcrumb segments as `Vec<&'static str>` using `Box::leak` for static lifetimes
- [x] Updated `Breadcrumbs::new` signature to accept `Vec<String>` and store segments directly without extra conversion
