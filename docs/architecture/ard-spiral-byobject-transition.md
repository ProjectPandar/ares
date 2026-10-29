# ARD — by-object spiral per-object transition_out

Non-negotiable decisions (spec: `spiral-byobject-transition-spec.md`).

1. **Per-batch literal semantics.** `final_layer` is
   `layer_index + 1 == object_layer_counts[object_index]` — mirroring
   `GCode.cpp:3805-3806`'s per-object-batch `last_layer` under
   by-object and degenerating to plate-final under by-layer spiral
   (single object by `Print.cpp:1334-1341` admission). No other
   condition, no new flag.
2. **Consumer enumeration is the contract.** The spec's table lists
   every `is_last_entry`/`last_entry` consumer; the only consumption is
   the `final_layer` gate. GREEN removes the orphaned plumbing
   (#1–#3) — no dead parameters, no compatibility shims.
3. **Deferred stays deferred.** The M73-placement and final-layer
   ramp-feed classes are out of scope; a GREEN that touches either is a
   STOP.
4. **Native-first RED, unweakened.** Runtime Native official + seeded,
   double-replay determinism, complete comparator, frozen input SHA;
   the witness reuses the committed admission fixture; no Native
   artifacts committed.
5. **Side-branch isolation.** The chain lives on
   `spiral-byobject-transition` from the W1 tip `a1a72362a`; it is not
   merged into the frozen window tip and not pushed; appending is a
   separate COORD decision after the RR.
6. **E2E-only verification.** No unit tests; the RED cases are the sole
   verification.
7. **Boundary hygiene.** GREEN touches `layers/object_entry.rs` (the
   expression) plus the orphaned plumbing removal across
   `layers/schedule.rs`, `layers.rs`, `layers/tool_entry.rs`,
   `layers/drive.rs`; queued and W1-touched files are otherwise
   untouched.
