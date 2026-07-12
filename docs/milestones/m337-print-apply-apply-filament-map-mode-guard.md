# M337: PrintApply apply filament_map_mode guard

## Source boundary

Source boundary is `OrcaSlicer/src/libslic3r/PrintApply.cpp:1190-1192`: after diff collection, `Print::apply(...)` begins BBS filament-map processing by copying `print_diff` into `print_diff_set` and entering the following block only when `filament_map_mode` is absent from that set.

```cpp
//BBS: process the filament_map related logic
std::unordered_set<std::string> print_diff_set(print_diff.begin(), print_diff.end());
if (print_diff_set.find("filament_map_mode") == print_diff_set.end())
```

Supporting context is the staged `print_diff` result identity at `OrcaSlicer/src/libslic3r/PrintApply.cpp:1184` and the downstream branches at `PrintApply.cpp:1194-1228`, which are intentionally deferred. This milestone models only set construction and the guard decision as private staged data.

## Exit criteria

- Preserve the comment intent `BBS: process the filament_map related logic`.
- Preserve set identity `print_diff_set` derived from `print_diff`.
- Preserve unordered-set semantics for membership and size, including duplicate suppression.
- Preserve guard key `filament_map_mode`.
- Preserve entering downstream filament-map processing only when `filament_map_mode` is absent from `print_diff_set`.
- Preserve not entering downstream filament-map processing when `filament_map_mode` is present.
- Keep all new Rust symbols private to `ares-core` staged `print_apply` modules.
- Defer map-mode lookup and `< fmmManual` branch from `PrintApply.cpp:1194-1204`, manual branch from `PrintApply.cpp:1205-1226`, reassignment of `print_diff` from `PrintApply.cpp:1227-1228`, real `unordered_set` integration, real config mutation, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.
