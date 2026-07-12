# M342: PrintApply apply print_diff set reassignment

## Source boundary

Source boundary is `OrcaSlicer/src/libslic3r/PrintApply.cpp:1227-1228`: after filament-map processing may mutate `print_diff_set`, `Print::apply(...)` compares `print_diff_set.size()` with `print_diff.size()` and reassigns `print_diff` from `print_diff_set.begin()` to `print_diff_set.end()` only when the sizes differ.

```cpp
if (print_diff_set.size() != print_diff.size())
    print_diff.assign(print_diff_set.begin(), print_diff_set.end());
```

Supporting context is the staged `print_diff_set` rewrite path from M337-M341. This milestone models only the size comparison and conditional reassignment as private staged data. The upstream container is `std::unordered_set`, so reassigned order is not a semantic guarantee; Rust staging must not promise preservation of the original `print_diff` order after reassignment.

## Exit criteria

- Preserve the exact size comparison gate: reassignment occurs only when staged set size differs from original `print_diff.size()`.
- Preserve no reassignment when sizes are equal, even if membership/order differs.
- Preserve duplicate suppression before reassignment through staged set semantics.
- Preserve reassignment from the staged set contents when sizes differ.
- Do not expose or depend on stable output order as part of the source semantics.
- Keep all new Rust symbols private to `ares-core` staged `print_apply` modules.
- Defer apply-status handling from `PrintApply.cpp:1231-1239`, lock acquisition from `PrintApply.cpp:1241-1242`, real config mutation, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.
