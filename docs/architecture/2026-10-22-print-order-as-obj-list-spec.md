# SPEC: `print_order = as_obj_list` object-instance ordering (ByLayer)

Status: implementation `87aeb0b6c` passes the four targeted cases in both modes.
Strict dual-mode FULL acceptance is not yet achieved; RED `dd750b6eb` remains unchanged.

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
- **Authorized bounded reduction (coordinator review).** The `init_order` pass sorts by model object
  index and renumbers to unique `1..N` keys. Subsequent `lower_bound` collection and re-sort preserve
  that assigned order. For a total sort range of `n <= 16`, libstdc++ uses insertion sort, preserving
  the input iteration order among equal model-object keys. The accepted Rust reduction therefore
  orders by model object index and preserves same-object instance iteration order in that range.
  For `n > 16`, the upstream relative order of same-object instances has **not** been ported.
  No parity claim or new tie-breaking contract is made for that case; stable Rust ordering outside
  the bounded range is not evidence of upstream equivalence.
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

The complete source explanation is `GCode.cpp:4960/4964/5059`, which passes `layers.size()` as
`num_objects` to `object_by_extruder` (`:4095-4101`). Every filament's vector therefore contains
rows for **all layer objects**, including empty rows. The `sorted` table at `:4140-4149` only
checks `layers[layer_id].original_object`, not islands. In contrast, Default filters empty islands
and absent/empty support at `:5116-5120` before chaining. AsObjectList looks up the global ordering
against the complete table, so foreign-filament objects yield empty visits. Shared PrintObject
identity is not required; the earlier inference that it was required is retracted.

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
- The validated ByLayer `init_order` effect described in §2; `Print.cpp:934` makes it reachable before
  the export call with `init_order=false`.
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
- Sequential-clearance assignment of `arrange_order` remains deferred. ByLayer initialization is
  included through the bounded reduction in §2, not deferred as unreachable.
- Upstream same-object instance tie order for total range `n > 16` remains unported.

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

## 6. Explicit ordering state and sequence boundary

`Instance.arrange_order` owns the initialized model-order rank (`1..N`); `position` remains
only the existing bucket identity/nearest-neighbour scheduling key. AsObjectList emission sorts
by `arrange_order`, not `position`.

`collect` obtains `(source_index, transform_index)` from the prepared object's identity.
The initial sort key is **source_index**, the index in `project.objects()`, not the ordinal of
the split print object in `traversal.objects`. `effective_config/grouping.rs` groups transforms
per source ModelObject and retains original instance membership within each transform group;
`candidates.rs` carries `source_object_index` into the resolved object. Thus differently transformed
PrintObjects from the same ModelObject retain equal initial keys before rank assignment.
The stable equal-key reduction preserves the existing PrintObject/instance traversal order within
the authorized `n <= 16` range; the limitation for larger ranges remains as specified in §2.

Both rank selection and per-filament empty-visit admission require `print_sequence == ByLayer`
and `print_order == AsObjectList`. ByObject keeps its pre-existing scheduling path even when the
project carries `print_order=as_obj_list`; this slice does not assign sequential-clearance ranks.

Exit criterion for GREEN: all four tests pass, with the two controls unchanged, and no change to
`golden::compare_ordered_bytes_generator_only`. Acceptance also requires the coordinator's strict
dual-mode neighbour and FULL gates, with complete logs and actual process exit statuses.
