# Spec: flush_into_objects object-wipe semantics — wiping marks and layer tool lists

Status: DRAFT (awaiting review) — discovery amendment recorded below
Base: `0266b8ef470384465587d52c8a0fd9dded84d98a`
Upstream boundary: `ToolOrdering.cpp` `WipingExtrusions::is_overriddable`
(:1586-1596), `mark_wiping_extrusions` (:1613-1755, dedicated-object
ordering :1627-1636, perimeters-then-infill passes :1641-1700),
`ensure_perimeters_infills_order` (:1763-1826), and the LayerTools
extruder-list construction in `ToolOrdering::compute`
(`ToolOrdering.cpp:197-240`, `handle_dontcare_extruder`) that the
marking feeds.

## Witness (probe, native first, CPU7,8, frozen input)

`pf-obj2` = R1 SEMM base (prime tower, 2 filaments, by-layer) with
`skirt_type=perobject`, `flush_into_objects=1` (both objects):

- Native: rc=0. `; filament: 1`, `; filament used [mm] = 500.91,`
  (second extruder unused) — every extrusion of the tool-2 object is
  overridden to tool 1, so the print collapses to a single tool: zero
  toolchanges (the one M600 match is inside the config block), no
  prime tower extrusions, 50 monotonic layers, skirt 2/2.
- Ares (guard term lifted, probe only): rc=0 but `; filament: 2,1`,
  both tools per layer, 25xT1 + 26xT0 + 52 M600, tower-first layer
  order — normalized divergence 21851 lines / 310 hunks; the first
  divergence is the header `; filament:` line itself.

So the divergence is NOT skirt geometry (skirt blocks are 2 vs 2); it
is the object-wipe marking semantics and their effect on the layer
tool plan.

## Discovery result (collapse owner located)

`ToolOrdering::collect_extruders` (`ToolOrdering.cpp:660-760`) is the
collapse owner: while building `LayerTools.extruders` it gates every
wall/fill extruder push on `something_nonoverriddable`, computed by
`WipingExtrusions::is_overriddable_and_mark` over the layer's
perimeter collections (:688-698) and fill collections (:718-733).
With `flush_into_objects` (all roles overriddable) and a by-layer
print (`m_print_config_ptr` set), a fully-overriddable region adds NO
extruder to the layer tool list — so the tool-2 object contributes
nothing and the print collapses to one tool with no toolchange/tower.

Ares counterpart: `gcode_emit/tool_ordering/collect.rs::object`
pushes wall/infill extruders unconditionally and never consults the
wiping overridable marking (only `wiping/assign.rs::mark` uses
`wiping/source.rs`'s overridable, at tower-planning time). The fix
lands in `collect.rs` (mirroring the `something_nonoverriddable`
gating) using the existing `wiping/source.rs` overridable predicate —
the port is fixed in place, no parallel path. `assign.rs` semantics
stay as they are unless the probe shows a further residual after the
collect fix.

## Included

1. Discovery step (mandatory, before code): localize where the
   single-tool collapse happens upstream — whether
   `mark_wiping_extrusions` consuming the full flush volume removes
   the extruder from `LayerTools.extruders` during
   `ToolOrdering::compute`, or `handle_dontcare_extruder`
   (:189-240) reorders, or the emission skips toolchanges whose
   entities are fully overridden. The SPEC scope splits by that
   answer: the marking port in `tool_ordering/wiping/`, or the
   layer-tool-list builder in `tool_ordering/` (`collect.rs` /
   `partitions`), or both.
2. Port whichever owner(s) the discovery names, into
   `crates/ares-core/src/project_slice/gcode_emit/tool_ordering/`
   (`wiping/assign.rs`, `wiping/source.rs`, `collect.rs` as needed).
   ares already carries a `mark` port (assign.rs, citing
   `ToolOrdering.cpp:1620-1755`) whose observable behavior diverges —
   the discovery must also determine whether the port itself is wrong
   or never consulted for the tool lists.
3. GREEN retires the last `flush_into_objects` disjunct in
   `skirt/generate.rs` (the disjunct and its comment go away
   together).
4. Public RED: single leg on `pf-obj2` (native ACTIVE assertion: the
   second extruder's `; filament used [mm]` entry is empty / the
   header lists one filament), strict parity; today Ares rejects via
   the guard, so the leg FAILs before the GREEN.

## Deferred

- Nothing else remains in the per-object skirt guard.

## Exit criteria (blast radius: whole wiping-marking + tool-ordering path)

- RED FAIL at the RED hash, PASS at the GREEN hash.
- Full tower/wipe/toolchange/skirt families + native_bbl, both modes,
  at the exact GREEN hash.
- fmt/clippy/wasm clean; no file over 400 LOC.
