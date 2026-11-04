# SPEC — `needs_retraction` internal-region suppression on the ROUTED travel polyline

Status: SPEC only (no implementation). Read-only discovery preceded this SPEC; the GDB evidence in §2 is what fixes the boundary.

## 1. Upstream boundary (source of truth)

Owning translation unit: `OrcaSlicer/src/libslic3r/GCode.cpp`.

| Upstream | Location | Role in this slice |
| --- | --- | --- |
| `GCode::travel_to(const Point&, ExtrusionRole, std::string, double)` | `GCode.cpp:7350` | Caller. Builds the straight `Polyline travel{last_pos(), point}` (`:7356`) and calls the predicate. |
| `GCode::needs_retraction(const Polyline&, ExtrusionRole, LiftType&)` | `GCode.cpp:7528` | The decision function being ported. |
| short-travel gate | `GCode.cpp:7530-7533` | **Included** (already present in Ares). |
| force-retract on leaving an external perimeter | `GCode.cpp:7607-7616` | Adjacent; Ares approximates it via `previous_feature`. **Not changed by this slice.** |
| support island / tree base containment | `GCode.cpp:7618-7632` | **DEFERRED** (see §6). |
| **internal-region suppression branch** | `GCode.cpp:7634-7640` | **This slice.** |
| `RetractWhenCrossingPerimeters::travel_inside_internal_regions(const Layer&, const Polyline&)` | `GCode/RetractWhenCrossingPerimeters.cpp:9` | Geometric predicate used by the branch. |
| routed re-check | `GCode.cpp:7421-7427` | **This slice.** `travel_to` replaces `travel` with the avoid-crossing detour and calls `needs_retraction` **again** on that routed polyline. |

The two calls, verbatim:

```cpp
// GCode.cpp:7359 — first evaluation, straight travel
bool needs_retraction = this->needs_retraction(travel, role, lift_type);
...
// GCode.cpp:7415-7427 — routing, then RE-EVALUATION on the routed polyline
if (m_config.reduce_crossing_wall
    && !m_avoid_crossing_perimeters.disabled_once()
    && m_writer.is_current_position_clear())
{
    travel = m_avoid_crossing_perimeters.travel_to(*this, point, &could_be_wipe_disabled);
    needs_retraction = this->needs_retraction(travel, role, lift_type);
}
...
// GCode.cpp:7433 — the FINAL value is the one that decides
if (needs_retraction) { ... retract ... } else { m_wipe.reset_path(); }
```

```cpp
// GCode.cpp:7634-7640
if (!is_perimeter(role) && m_config.reduce_infill_retraction && m_layer != nullptr &&
    m_config.sparse_infill_density.value > 0 &&
    m_retract_when_crossing_perimeters.travel_inside_internal_regions(*m_layer, travel))
    return false;   // suppress retraction
```

`travel_inside_internal_regions` is a pure geometry query: it caches, per `Layer`, the
`surface.expolygon` of every `surface.is_internal()` across `layer.regions()`, builds an AABB tree over
their bounding boxes, and answers whether the **travel polyline** lies inside one internal island
(`GCode/RetractWhenCrossingPerimeters.cpp:9-40`).

## 2. What the read-only GDB pass established (this is why the boundary looks like this)

Binary provenance: fresh `cargo build -p ares-cli` in worktree at
`3fbad615e334a283596d0e6acff921f1d02804c6`, clean tree,
`sha256 = dbc1c5048d2c3ea9fc7661a9b967f71b484edf46877813b80e28a1eb06eb1519`.

Ares side, breakpoint at `can_skip_retraction` (`motion/path/retraction.rs:12`) for the travel that
`tests/parity/arachne/default-prisms.3mf` emits at G-code line 1170 (destination `X126.965 Y135.187`):

```
reduce_infill_retraction = true
has_sparse_infill        = true
previous_feature         = Some("Internal solid infill")
current_is_perimeter     = false
inside_internal_surface  = false   <-- the ONLY failing conjunct
```

Native side, breakpoints on `GCode::needs_retraction` in the official 2.4.2 binary (dynamic symbols only;
arguments read from the SysV registers `$rsi` = `&travel`, `$edx` = `role`), selecting the same travel by
its origin-independent displacement `(+5805706, +8374950)` coord_t:

```
role=5 (erSolidInfill), travel n=2 (straight)  -> needs_retraction returns 1   (retract)
role=5 (erSolidInfill), travel n=6 (routed)    -> needs_retraction returns 0   (no retract)
```

Three objects in the fixture, three identical pairs. `is_perimeter(erSolidInfill)` is `false`
(`ExtrusionEntity.hpp:56-61`: only `erPerimeter`/`erExternalPerimeter`/`erOverhangPerimeter`), so
`!is_perimeter(role)` is `true` and the branch is reachable.

Call-count corroboration: Native enters `needs_retraction` **659** times for this fixture; Ares enters
`can_skip_retraction` **330** times. Native evaluates roughly twice per travel, Ares exactly once.

**Conclusion that fixes the boundary: the defect is NOT a missing predicate and NOT a role whitelist.**
Ares already has the perimeter blacklist (`current_is_perimeter`) and already has the geometric test
(`travel::inside_internal_surfaces`, wired at `start_travel/approach.rs:63-69`). The defect is that Ares
evaluates that geometry on the **straight** segment, before routing, and never re-evaluates it on the
routed polyline, whereas upstream's final answer comes from the second evaluation on the detour.

## 3. Ares destination boundary

| Ares | Location | Change |
| --- | --- | --- |
| `emit_needs_travel` | `crates/ares-core/src/project_slice/gcode_emit/motion/path/start_travel/approach.rs:31` | Compute `inside_internal_surface` **after** `plan_route` and **on the routed polyline**; keep a pre-route value only if upstream's first call needs it. |
| `can_skip_retraction` | `crates/ares-core/src/project_slice/gcode_emit/motion/path/retraction.rs:6-20` | Unchanged signature and conjunct order; the caller must feed it the routed-geometry answer. |
| `travel::inside_internal_surfaces` | `crates/ares-core/src/project_slice/gcode_emit/motion/travel.rs:260` | Must accept the routed vertex list, not only the `cursor → first_scaled` segment. It already takes `[start, destination]`; extend to the full `route` the way `inside_support_islands` (`retraction.rs:23-47`) already does. |
| `plan_route` | `.../start_travel/route.rs:9` | Already returns the detour; ordering only. |

Explicitly **not** in this slice:

- `crates/ares-core/src/gcode_travel_retraction.rs` (`should_retract`, `reduce_infill_retraction_applies`,
  `reduce_infill_retraction_role`). GDB proves these symbols are **never entered** during a CLI `slice`,
  so the "role whitelist / `previous_non_gap_fill_role`" conjuncts the earlier discovery pointed at are
  **not the live path**. They are out of scope here and are recorded as a separate cleanup question in §6.
- `wipe` / `Wipe::wipe` (`GCode.cpp:426`, `motion/travel/wipe_sequence.rs`). Downstream consequence; it
  follows automatically once the retract decision matches.

## 4. Included upstream behavior

1. Upstream's decision order inside the predicate: short-travel gate, then the internal-region branch,
   evaluated on the polyline that will actually be travelled (`GCode.cpp:7421-7427`).
2. The final `needs_retraction` value is the post-routing one (`GCode.cpp:7433`).
3. The internal-region test remains a geometric query over `surface.is_internal()` regions of the current
   layer, evaluated over the whole travel polyline including detour vertices.
4. `!is_perimeter(role)` stays a three-value blacklist (`ExtrusionEntity.hpp:56-61`), not an infill
   whitelist.

## 5. Deferred upstream behavior

1. **Support island / tree-base containment, `GCode.cpp:7618-7632`.** Ares has a partial equivalent
   (`inside_support_islands`, `retraction.rs:23-47`, used at `approach.rs:88-101`) operating on routed
   geometry; making it match upstream exactly is a separate slice.
2. Force-retract on leaving an external perimeter, `GCode.cpp:7607-7616`, currently approximated by the
   `previous_feature` conjunct (`retraction.rs:17`). Not retouched here.
3. `m_avoid_crossing_perimeters.disabled_once()` / `use_external_mp_once` state machine details beyond
   what routing already models.
4. The `Wipe::wipe` ordering item already recorded as DEFERRED in
   `docs/architecture/native-artifact-cleanup-arachne-prisms-spec.md:52,61`.
5. Whether `gcode.rs` + `gcode_travel_retraction.rs` are dead code for the CLI `slice` command (§6.1).

## 6. Open questions this SPEC deliberately does not answer

1. `gcode.rs:282` builds a `TravelRetractionCommand` and calls `retract_before_travel` for every move, yet
   gdb never enters `should_retract` in that module during a slice. Either that path is unreachable from
   `slice_project_plate_validated`, or it is compiled but unused. This needs its own read-only check
   before anyone "replaces the role-whitelist conjuncts" — doing so would edit code that never runs.
2. The fixture's internal-surface set: Ares' `island_print_order::internal_surfaces` versus upstream's
   `surface.is_internal()` over `layer.regions()`. Equal on this fixture's straight evaluation (both say
   "not inside"), but equality on the routed polyline is exactly what the RED must measure.

## 7. Exit criteria for the GREEN that follows this SPEC

1. `default-prisms.3mf` byte-strict parity with official Native under
   `native_stream_case::check` (currently RED).
2. The `reduce_crossing_wall=0` sole-key control stays byte-strict (proves the fix is gated on routing and
   does not change the unrouted path).
3. No change to any other existing parity case.
4. The support-island branch (§5.1) untouched, so its behavior stays byte-identical.

## 8. Supplement — the external boundary is missing the per-instance `translate(instance.shift)`

Status: SPEC supplement, added after §7's GREEN. The 7 difflib hunks that §7 left behind in
`default-prisms.3mf` are **not** retraction hunks: they are avoid-crossing **detour waypoints** on
hole→neck inter-object travels. This section owns them.

### 8.1 Upstream boundary (source of truth)

| Upstream | Location | Role |
| --- | --- | --- |
| `AvoidCrossingPerimeters::get_boundary_external` | `AvoidCrossingPerimeters.cpp:1137-1193` | Builds `m_external`: every print object's `lslices` holes at this print z. |
| per-instance translation | `AvoidCrossingPerimeters.cpp:1172-1177` | For each `PrintInstance`, append `holes_per_obj` and then `boundary[i].translate(instance.shift)`. **This is the missing step.** |
| `travel_to` routing frame | `AvoidCrossingPerimeters.cpp:1241-1242` | `scaled_origin = use_external ? Point::new_scale(origin) : Point(0,0)`; `start = last_pos + scaled_origin`, `end = point + scaled_origin`. |

The live function set is the one §2 of the earlier discovery established: `#if 0` starts at `:1340`, so
`get_boundary_external :1665`, `travel_to :1549` and `avoid_perimeters :1521` are dead copies. Only
`:1137-1193`, `:1233-1316` and `:691` are compiled.

### 8.2 What the read-only GDB pass established

Travel identified on both sides by its origin-independent displacement: `(+17708658,+25166913)` →
`(+3038330,+756944)` coord_t, the hole→neck travel at G-code line 1331.

| | Native (`m_external`, offset `0x1d0` in `this`) | Ares (`boundary.contours`) |
| --- | --- | --- |
| polygons | 1 | 1 |
| vertices | 4 | 4 |
| geometry | rect `(127207833,127792164) (120792170,127792164) (120792170,134207827) (127207833,134207827)`, 6.41572 mm square | rect `(3207832,-3207831) (-3207831,-3207831) (-3207831,3207832) (3207832,3207832)`, 6.4157 mm square |
| position | centred at plate `(124.000, 131.000)` | centred at `(0, 0)` of the router frame |

Count, size and vertex order agree. Only the **position** differs, by exactly `(9.5617, 17.0000) mm`.

The fixture's instance transforms (`3D/3dmodel.model` build items) name the two frames:

| object | `transform` translation |
| --- | --- |
| `hole.stl` (`objectid=2`) | `(124.000001, 130.999995)` |
| `neck.stl` (`objectid=6`) | `(114.438309, 113.999995)` |
| `wide.stl` (`objectid=4`) | `(104.000000, 131.000005)`, rotated |

Native's centre `(124.000, 131.000)` **is** `hole.stl`'s placement, i.e. instance-translated. Ares' centre is
object-local `(0, 0)`; the displacement is exactly `hole.shift - neck.shift = (9.5617, 17.0000)`, because
`neck.stl` is the object currently being printed and only *its* shift is subtracted from the endpoints.

### 8.3 Ares coordinate frames and the conversion

Two frames are in play, and the router already commits to one of them.

- Producer side (`layers/group.rs:35-59`) flattens `object.slices(index)` for every entry. `Layer::lslices`
  is per-print-object **object-local** (`GCode.cpp:5380/5403/5437` applies the placement later via
  `set_origin(unscale(instance.shift))`), so `chunk_slices` is a mixture of local frames.
- Router side (`start_travel/route.rs:27-31`) builds `start` as `unscale(cursor) + state.offset`, i.e.
  **plate** coordinates, and `avoid_crossing::route` immediately converts back with
  `to_scaled = checked_scale_rounded(point - offset)` (`avoid_crossing.rs:73-84`). The router therefore
  compares everything in the **currently printing instance's local scaled frame**.

The external boundary must be expressed in that same frame. Per hole, with `s = instance.shift` of the
object that owns the hole and `o = state.offset` of the instance being printed:

```
local_scaled = hole_local_scaled + s - o
```

This is the same relation upstream gets for free: upstream stores `m_external` in plate coordinates
(`:1172-1177`) and routes with plate endpoints (`:1241-1242`); Ares keeps its existing single local frame and
folds `-o` into the boundary input. Applied to the measured travel it reproduces Native exactly: the rect
becomes `x 6.3538..12.7694, y 13.7922..20.2078`, the detour corner from `(17.708658,25.166913)` to
`(3.038330,0.756944)` is `(12.7694, 13.7922)`, and Native's `wp1` is `(12769594,13792099)`.

`offset` is constant for the lifetime of the boundary cache: `state.avoid_boundary` is cleared per entry
(`layers/tool_entry.rs:162`, `layers/support_entry.rs:94`, `motion/state.rs:273`), and an entry is one
instance. So the shift may be baked into the boundary input and the `-o` term applied once at build.

**Multi-instance:** upstream appends one translated copy per `PrintInstance`, not one per object
(`:1172-1177`). Ares must do the same — an object placed twice contributes its holes twice, at both
placements. The chunk-wide slice list must therefore be expanded by `instances[object_index]`, which already
carries the scaled `shift` (`layers/instances.rs:9`, built from `footprint::instance_shift`, the analogue of
upstream's `instance.shift`).

### 8.4 Ares destination boundary

| Ares | Location | Change |
| --- | --- | --- |
| `group::geometry` | `layers/group.rs:35-59` | **Unchanged.** Still produces the object-local `chunk_slices` that brim/skirt/tower consume. |
| `group::boundary_slices` (new) | `layers/group.rs` | Per entry, per instance of that object, append the layer slices translated by `instance.shift`. Plate-frame, chunk-wide, identical for every instance in the chunk. |
| `AvoidCrossingGeometry::chunk_slices` | `motion/state.rs:242` | Receives the `boundary_slices` vector. Field name and all six unit-test constructors unchanged; the doc comment states the new contract (`:1172-1177` applied). |
| `Boundary::build_external` | `motion/path/avoid_crossing/boundary.rs:47-52` | Takes the frame translation and applies `point + translate` while it is already walking the hole points (`:59-67`). |
| `avoid_crossing::route` | `motion/path/avoid_crossing.rs:136-171` | Computes the translation from `offset` and passes it to `build_external`. |

Nothing else moves: `plan_route`, `router::avoid_perimeters`, the internal boundary, the safe zone, the
detour cap and the retraction decision from §3 all keep their current shape.

### 8.5 Included upstream behavior

1. `get_boundary_external`'s per-instance translation (`:1172-1177`), one appended copy per instance.
2. Holes taken from every object with a layer at this print z, expanded by half the cross-object average
   perimeter spacing, miter join (`:1187-1193`; Ares `boundary.rs:47-118`, already matching — the measured
   rectangle sides agree to the unit).
3. The routing frame relation of §8.3, so the detour is computed against the boundary where the object
   actually sits.

### 8.6 Deferred upstream behavior

1. **`chunk_slices` consumers.** `brim_emit` (`layers/brim_emit.rs:22`) and `tower` (`layers/tower.rs:18`)
   still receive object-local slices. Whether that is a latent defect for them, or whether they re-apply the
   placement elsewhere, is **not investigated and not changed here**. If it is
   a defect, it is a separate slice with its own RED.
2. **`m_external` bbox invalidation.** Upstream re-checks
   `!(bbox.contains(start) && bbox.contains(end))` per travel (`:1266-1272`); Ares builds the external
   boundary once per entry (`route.rs:38`) and only the internal boundary has a bbox test
   (`avoid_crossing.rs:103-104`). Included only if a residual difference after §8 is traced to it.
3. The object set stays chunk-scoped (`entries`) rather than print-wide
   (`layer.object()->print()->objects()`). Identical on this fixture (all three objects are in one chunk);
   not changed here.

### 8.7 Exit criteria for the §8 GREEN

1. `default-prisms.3mf` byte-strict including the M73 handling (the comparator is **not** touched).
2. The `reduce_crossing_wall=0` sole-key control stays byte-strict.
3. `arachne` suite and the avoid-crossing / `reduce_crossing_wall` / multi-object / multi-instance parity
   cases show no regression.
4. `cargo fmt` and `cargo clippy --all-targets` clean.
