# SPEC: `print_order = as_obj_list` object-instance ordering (ByLayer)

Status: SPEC only. RED inputs are committed with this document; no GREEN code yet.

Source-cited rewrite slice of `libslic3r`. Upstream owner first; Ares is the port target.

## 1. Upstream boundary

Four sites, all in `OrcaSlicer/src/libslic3r`:

1. `GCode.cpp:2392-2430` — `sort_object_instances_by_model_order(const Print&, bool init_order)`.
   - `:2404-2412` build `vector<pair<const ModelInstance*, const PrintInstance*>>` by iterating
     `print.objects()` × `print_object->instances()`; with `init_order` it also assigns
     `arrange_order = find_object_index(model, print_object->model_object())`.
   - `:2413` `std::sort` on `l.first->arrange_order < r.first->arrange_order`.
   - `:2415-2418` with `init_order`, renumber `arrange_order = k + 1` so each instance gets a unique order.
   - `:2422-2428` collect: iterate `print.model().objects` × `model_object->instances`, and for each
     `model_instance` do `std::lower_bound` over the sorted pairs **keyed on `arrange_order`**, emitting
     the match; then `std::sort` the result again by `arrange_order`.
2. `GCode.cpp:2791-2796` — `print_object_instances_ordering =`
   `print_order == PrintOrder::Default ? chain_print_object_instances(print) : sort_object_instances_by_model_order(print)`.
   Called unconditionally for both print sequences (this block runs before the
   `print_sequence` split). `init_order` is defaulted (false) at this call site.
3. `GCode.cpp:5125-5133` — per-filament instance selection:
   - ByObject: `sort_print_object_instances(objects_by_extruder, layers, ordering, single_object_instance_idx)`
     — passes `ordering` unconditionally.
   - ByLayer: `ordering_for_filament = (print_order == PrintOrder::AsObjectList && ordering != nullptr)
     ? ordering : &new_ordering` where `new_ordering = chain_print_object_instances(print_objects, &wt_pos)`
     reversed (`:5124-5125`). This is the only place `print_order` is consulted for instance order.
4. `GCode.cpp:4118-4167` — `sort_print_object_instances(...)`: with `ordering != nullptr` it builds
   `(PrintObject*, ObjectByExtruder*)` pairs from `layers[i].original_object`, `std::sort`s them by
   pointer, then emits one `InstanceToPrint` **per entry of `ordering`** (in `ordering` order) whose
   `PrintObject` is found in that sorted vector.

Marker emission is already correct in Ares and is not part of this slice: `; printing object`
`GCode.cpp:5349` (gated on `gcode_label_objects`, before the island loop), `; stop printing object`
`GCode.cpp:5472` (after the island loop at `:5434`; emitted even when
`object_by_extruder.islands` is empty).

Supporting state: `int arrange_order = 0;` `Model.hpp:1264`.

## 2. Determinism of `arrange_order` — measured, not assumed

An earlier revision of this SPEC claimed every key is `0` in ByLayer slicing, on the strength of the
sole assignment at `Print.cpp:881`. GDB observation of the official binary retracted that. What is true:

- **Not persisted.** `arrange_order` is mentioned only in `Print.cpp` and `GCode.cpp`. There are zero
  references in `Format/bbs_3mf.cpp` or `Format/3mf.cpp`. A project loaded from a 3MF therefore starts
  with every `arrange_order == 0` (`Model.hpp:1264` initializer).
- **Two writers, and the ByLayer one does fire.**
  - `Print.cpp:881` in `Print::sequential_print_clearance_valid` (`Print.cpp:613`) — the by-object path.
  - `GCode.cpp:2409-2410`, reached because `layered_print_cleareance_valid` (`Print.cpp:932`) calls
    `sort_object_instances_by_model_order(print, /*init_order*/ true)` at `Print.cpp:934`. That validator
    is the **by-layer** arm of `Print::validate` (`Print.cpp:1306`, the `else` of the sequential branch),
    so it runs for our RED inputs. With `init_order == true` the function assigns
    `arrange_order = find_object_index(model, print_object->model_object())` — the ModelObject's index in
    `model.objects`, `-1` when absent — and then (`:2415-2418`) renumbers the sorted result to a unique
    `1..N`.
- **Measured consequence.** With the two-instantiated RED fixture, breaking on
  `Slic3r::sort_object_instances_by_model_order` shows two CLI calls: one with `init_order=1` (from
  `Print::validate` → `layered_print_cleareance_valid`) and one with `init_order=0` (`GCode.cpp:2796`);
  both return **2 unique `PrintInstance*`, in the same order**, and the returned pointers are not
  duplicated. So `print_object_instances_ordering` is not where multiplicity comes from, and the
  comparator input is *not* all-equal — after the `init_order=true` pass the keys are a unique `1..N`
  derived from the model object list.
- **What is still implementation-defined.** Ties remain wherever several instances share one ModelObject
  index, and the renumbering order among such ties comes from `std::sort` (`:2413`), which is not stable.
  That is the only place where a libstdc++ introsort artifact could leak into the instance order, and it
  only matters for objects with more than one instance.
- **Therefore the SPEC still forbids inventing an ordering rule.** Ares must port the algorithm literally
  (assign `arrange_order` from the model object index in the `init_order` pass, renumber `k+1`, sort with
  the same comparator, collect in the same iteration order with the same `lower_bound`, re-sort) rather
  than deriving an order from the object list directly. It may **not** substitute `sort_by_key`+stable or
  any hand-rolled "object list order" on the grounds that it looks equivalent.
- The RED below pins the resulting bytes, so an implementation that quietly picks a different
  permutation fails.

Settled by the same GDB round: **where the multiplicity comes from** (§4 measures the effect, this
locates the cause). The two candidate explanations in the previous revision are both rejected:
(a) is impossible because the keys are a unique `1..N`, not all-zero, so `lower_bound` at `:2424` does not
collapse to `begin()`; and (b) is directly falsified — the returned vector holds 2 unique pointers.
What the measurement does show is that the doubling needs **two filaments**: forcing both objects onto
filament 1 makes `print_order=as_obj_list` produce 100 marker pairs over 50 layers, i.e. 2 per layer,
exactly like the Default path (no divergence at all). With the shipped two-filament layout it is 4 per
layer. So the extra visits are produced by the per-filament loop (`:5093-5140`) emitting both instances
for *each* filament — the `ordering_for_filament = ordering` select feeding the `ordering != nullptr`
branch of `sort_print_object_instances` (`:4118-4167`), whose `original_object` lookup (`:4137`, `:4153`)
admits instances that belong to the other filament's object. That is the behaviour to port, and the
single-filament variant is a cheap way to prove the port is aimed correctly during GREEN.


## 3. Ares destination boundary

- `crates/ares-core/src/project_slice/gcode_emit/layers/object_order.rs` — currently holds only the
  Default path: `chain_instance_order` (`:5`) and `chain_without_start` (`:9`), both over
  `chain_segments_constrained_with_optional_start`. It gains the `sort_object_instances_by_model_order`
  sibling.
- `crates/ares-core/src/project_slice/gcode_emit/layers/instances.rs` — carries instance identity; must
  also carry `arrange_order` (Ares has no field for it today: `grep arrange_order crates/` = 0 hits).
- the ordering selection site (wherever `chain_instance_order` is currently called for the layer loop) —
  reads `print_order` and picks Default vs AsObjectList, i.e. the port of `GCode.cpp:2791-2796`.
- the per-filament chunk path that emits `InstanceToPrint`-equivalent entries — gains the
  `ordering_for_filament` select of `GCode.cpp:5129-5133` and the `ordering`-driven branch of
  `sort_print_object_instances` (`GCode.cpp:4118-4167`).
- `print_order` already parses (`crates/ares-core/src/options/process_options/print_source.rs:23`,
  metadata `registry/definitions/metadata.rs:12884-12890`, enum `["default", "as_obj_list"]`) and has
  **zero consumers** in `project_slice/` today; `island_print_order` is unrelated intra-object ordering.

## 4. Included / deferred

Included (this slice):
- `print_order` read at the ByLayer instance-order selection.
- literal port of `sort_object_instances_by_model_order` including the `init_order` branch, even though
  the CLI ByLayer call site passes the defaulted `false`.
- literal port of the `ordering != nullptr` branch of `sort_print_object_instances`.
- `arrange_order` carried on the Ares instance model, defaulting to 0, never persisted (matching the
  3MF facts in §2).

Deferred (named so they are not silently absorbed):
- **ByObject path** (`GCode.cpp:5125-5127`). There `ordering` is passed unconditionally and
  `arrange_order` *can* be non-zero, but only because `Print::sequential_print_clearance_valid`
  (`Print.cpp:613/-881`) ran — a separate upstream boundary (sequential-printing clearance) that Ares
  owns elsewhere. Measured control: `byobject-perobject-multi-copy-brim/combined-brim.3mf`
  (`print_sequence = by object`) with `print_order = as_obj_list` produces **zero** divergence
  (60/60 markers, 0 diff lines), so there is nothing to pin in this slice.
- Any `arrange_order` *assignment* policy. This slice only propagates the field; assigning non-zero
  values belongs to the sequential-clearance port.
- `init_order`-driven renumbering as an observable behaviour: ported but unreachable from the CLI ByLayer
  call site, so untested by the RED.

## 5. RED design and evidence

Harness: `runner::native_stream_case::check` — Native double replay (nondeterminism guard) plus
`NativeCase::compare`, i.e. the public API and the CLI both compared through
`golden::compare_ordered_bytes_generator_only`. Inputs are committed as 3MF + SHA only; no Native
constants and no marker masks.

Derived inputs (`tests/parity/print-order-as-obj-list/`), all deterministic zips (fixed 1980-01-01
timestamps) from existing GUI-reachable fixtures:

| input | derivation | sha256 | Native vs Ares on main `0266b3494` |
|---|---|---|---|
| `class-default.3mf` | `config-float-header/class.3mf`, `print_order=default` | `02847e15…` | 0 diff lines, 100/100 markers |
| `class-as-obj-list.3mf` | same, `print_order=as_obj_list` | `39092694…` | **200 diff lines**, 200 native / 100 ares markers |
| `three-instance-default.3mf` | same + one extra `<item>` at y=128 | `4e240a27…` | 0 diff lines, 150/150 markers |
| `three-instance-as-obj-list.3mf` | same 3-instance model, `as_obj_list` | `6c0f60d3…` | **300 diff lines**, 300 native / 150 ares markers |

The controls are clean on main and stay as regression guards. Multiplicity scales linearly with
instance count (2 instances → 200, 3 instances → 300 diff lines; 100/150 markers from Ares in both
cases), so the divergence is not a constant or a marker-count fixture artifact.

Direction of the divergence, measured: with the two marker comment kinds stripped, the `as_obj_list`
outputs are byte-identical (`body = 0` diff lines on every fixture tested). Ares' extrusion content and
order are already right; what is missing is the extra instance visits that upstream performs, each of
which emits `; printing object` / `; stop printing object` with an empty island list
(`GCode.cpp:5434` loop with `object_by_extruder.islands` empty). Per layer: 2 instances → 2 pairs
(default) vs 4 pairs (`as_obj_list`); 3 instances → 3 vs 6.

Exit criterion for GREEN: all four tests pass, with the two controls unchanged, and no change to
`golden::compare_ordered_bytes_generator_only`.
