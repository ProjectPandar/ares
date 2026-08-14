# Task 22O.95 — island print phase order

Port pinned `GCode.cpp:5434-5470,6131-6148` for one region. Flatten O94 island
ownership into perimeter/fill/thin print entities. Force wall-first on layer 0;
on later layers dispatch by the 3MF-derived `is_infill_first` option, preserving
within-phase order.

Focused tests cover first/later-layer option behavior, exact KSR ordered entity
inventory, repeatability, disposal, and public lifecycle. Separate modules,
<400 LOC, no source-splitting macros.

Deferred: infill greedy chaining/reversal, multi-region/tool/wiping, motion,
G-code.
