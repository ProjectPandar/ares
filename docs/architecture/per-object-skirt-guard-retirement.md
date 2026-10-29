# ARD: retire the remaining per-object skirt admission guard

This is a `libslic3r` rewrite slice, not a new Ares pipeline. The owning
boundary is `Print::_make_skirt` and the corresponding G-code emission.
The approved curl-lifecycle chain is unchanged by this slice.

## Source boundary and Rust destination

Upstream references are under `OrcaSlicer/src/libslic3r`:

| Upstream owner | Rust destination under `project_slice/gcode_emit/` |
| --- | --- |
| `Print.cpp:2646-2696`, skirt height and object/support occupied hulls | `skirt/generate.rs`, `skirt/occupied.rs::items` |
| `Print.cpp:2794-2819`, all instance shifts and brim/support-brim areas | `skirt/occupied.rs::items` |
| `Print.cpp:2825-2896`, per-object fixed-point envelope grouping | `skirt/member_groups.rs::merge` |
| `Print.cpp:2898-2971`, `make_group_brims` contact components and shared owners | `brim/combined.rs::group_brims` |
| `Print.cpp:2973-2991`, install skirt/brim groups | `skirt/generate.rs` |
| `GCode.cpp:4456-4483,4486-4540,5334-5335`, group lookup, once-per-layer skirt, owned brim, then object content | `layers/object_skirt.rs`, `layers/brim_emit.rs`, `layers/tool_entry.rs` |

These producers and emitters are already ported. The remaining admission
predicate in `skirt/generate.rs` freezes their combined-brim path to a
single-instance, single-filament, by-layer subset. Delete that predicate
instead of adding another implementation or a fallback. Existing Ares
containers remain compatibility shells around the upstream ownership and
ordering contracts, not independent skirt policy.

Keep the separate upstream rejection: `Print.cpp:2551-2556` throws
`SlicingError` when a ByObject per-object skirt is shared by multiple
PrintObjects (`Print.cpp:2977-2978`). Preserve that condition and message
through `SliceError::InvalidInput`, not `UnsupportedProjectFeature`.

## Non-negotiable decisions

- Consume the existing all-instance plate-frame occupancy and contact-brim
  producers. No new hull algorithm, global combined-skirt fallback, or
  extra skirt-distance expansion.
- Retain PrintObject group membership and once-emission tracking. Multiple
  instances of one PrintObject contribute to its single occupied hull.
- Keep geometry and emission in platform-neutral core code, with no native
  runtime, filesystem, terminal, or UI dependencies.
- Defer changes to individual brim algorithms, support/raft/tower
  generation, tool ordering, and sequential clearance validation. This
  slice retires admission over the existing source-owned path; it does not
  redesign adjacent algorithms.

## Native-first RED/GREEN evidence

Inputs live in `tests/parity/per-object-skirt-combine/`, with exact hashes
in `manifest.json`. They derive from the existing Ender-3 stock-profile
fixture, enabling combined brims and adding a second instance of one object.

- Far: two skirt groups and two brim blocks; the two instances share their
  PrintObject's occupied hull.
- Contact: one skirt group and one combined brim block.
- Two native invocations per input produce identical bodies after removing
  only the generator timestamp line.
- RED: both public API and CLI reject with the per-object skirt unsupported
  error. GREEN: strict whole-stream equality against Native, plus raw
  API/CLI equality. No Native outputs are committed.

The parity harness retains inputs, native launch/result/output, witness
counts, Ares output/errors, and strict comparison receipts under the
configured artifact root. Existing per-object skirt and brim controls must
remain unchanged. Full non-parity verification after the pending main
integration is a separate outstanding gate, not implied by scoped passes.
