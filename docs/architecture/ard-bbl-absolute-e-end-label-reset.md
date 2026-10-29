# ARD — end-label flush absolute-E reset

Upstream pin: `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
This is a source-cited writer rewrite slice, not an Ares pipeline feature.

## Non-negotiable boundaries

1. `GCodeWriter.cpp:1181-1192::add_object_end_labels` writes and clears a
   nonempty end-label buffer, then calls `reset_e(true)` for absolute E.
   `GCode.cpp:5699-5705` flushes labels before the layer Z move.
2. `GCodeWriter.cpp:480-502::reset_e(bool force)` owns flavor exclusions,
   the `is_zero(E) && !force` gate, counter reset, and absolute-E output
   with the existing comment handling. Extend the current Rust helper
   with `force`; existing callers pass false. Do not add another emitter.
3. Destination: `motion/state/object_labels.rs::append_exclude_end` calls
   that helper with true after either end-label variant is actually
   emitted. Drop the text-sniffing surrogate for counter reset. Existing
   label adapters are temporary compatibility shells, not owners of a
   separate reset rule.
4. No string insertion outside the shared writer helper, no reset on an
   empty pending slot, no E-zero suppression for a forced reset, and no
   re-enabling spiral layer-change retraction to hide a missing reset.
5. Included: emitted M625/end-text buffers, writer counter, and existing
   flavor/comment behavior. Deferred: unrelated start-label behavior,
   spiral taper/geometry, and general label redesign.

## Verification

The sole new RED is the A1 absolute-E two-object input. Its only setting
delta is `use_relative_e_distances=0`; Native and Ares both have 81 layers.
The Native boundary is `M625`, `G92 E0`, `G1 Z.4 F42000`; the complete
Native/API/CLI comparison must catch the missing reset and later E offset.
No unit tests or committed Native goldens. Register in a group, leaving
the root parity suite unchanged. After GREEN, run both Native modes and
existing label/reset/spiral neighbors, plus rustfmt and all-target clippy.
