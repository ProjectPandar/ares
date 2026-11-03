# Scarf default speed: one upstream small-perimeter decision

Status: worker6 source analysis and proposed spec for coordinator review.
No production or test changes; implementation requires spec approval and
worker1 integration. Browser gate is active; no heavy task was started.

## Source boundary

Pinned OrcaSlicer revision: `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
Read from `OrcaSlicer`, whose HEAD matches that revision.
All upstream paths below are relative to `OrcaSlicer/src/libslic3r/`.

| Source | Owning behavior |
|---|---|
| `GCode.cpp:5808–5813`, `GCode::extrude_loop` | Only default incoming speed (`-1`) permits small-loop selection; compare complete scaled loop length; zero resolves to half outer-wall speed; percent resolves against outer-wall speed. |
| `libslic3r.h:60–71,83–84` | Normal scaling factor is `0.000001`; `SMALL_PERIMETER_LENGTH(t)` is `((t / SCALING_FACTOR) * 2 * PI)`. Preserve this expression's units and ordering. |
| `ExtrusionEntity.cpp:49–52,174–179` | Path length comes from its polyline; loop length sums complete member polyline lengths. |
| `GCode.cpp:5893–5897,5950–5956` | Positive selected speed applies only to eligible nonbridge perimeters, including paths returned by `ExtrusionLoopSloped`. |
| `GCode.cpp:6515–6526` | Only when incoming speed is still `-1`, sloped inner/outer perimeter speed is `min(role_speed, scarf_joint_speed.get_abs_value(role_speed))`. No length comparison occurs here. |
| `GCode.cpp:6561–6595` | Zero-speed volumetric handling and layer overrides/ramping follow the speed selection. This slice does not rewrite them. |
| `ExtrusionEntity.cpp:426–509,511–524` | Sloped-loop construction splits geometry and returns starts/flats/ends; it does not choose small-perimeter speed. |
| `PrintConfig.cpp:2137–2156` | Small speed percentage base is outer-wall speed; threshold is a float in mm. |

Rust destination: existing platform-neutral
`crates/ares-core/src/project_slice/gcode_emit/motion/scarf.rs`, specifically
`build`'s default slope speed calculation and `scarf_speed`.
Its local speed field is an existing compatibility shell for the cited
`GCode::_extrude` sloped default, not a separate Ares speed policy.
No new crate, public API, I/O, UI, terminal or viewer behavior is needed.

## Current evidence and conclusion

Worker6 base is `3cad8b4c3a5ab52d38d853655ceef74d1b783b1f`.
`scarf.rs:64–68` sums per-path XY lengths after converting each to mm;
`:89` passes that length into `scarf_speed`; `:210–227` returns
`small_perimeter_speed` when the mm threshold passes, before resolving the
role-relative scarf speed and its cap.

The local threshold branch is structurally outside its upstream owner.
However, **small-loop speed bypassing scarf cap is correct upstream behavior**:
the loop caller already supplies positive speed, so `_extrude` skips the
entire default-role/scarf selection. A low scarf cap alone is not a valid
post-worker1 RED case.

Read-only worker1 context:

- `docs/architecture/gcode-small-perimeter-loop-speed-preselection.md` owns
  complete-loop caller selection and speed transport.
- `docs/architecture/gcode-small-perimeter-scarf-red-supplement.md` explicitly
  defers this local mm shortcut; its RED concerns caller speed and ramping.
- Pending `loop_paths/small_perimeter.rs` compares complete scaled length,
  resolves zero and filters positive speed. `loop_paths.rs` passes that
  selection to scarf members. Pending `features.rs` uses `slope.speed` only
  when requested caller speed is absent.

These pending files are observations, not an immutable approved base. Recheck
their actual merged revision before RED. Read-only SHA256 identities:

```text
scarf.rs (worker6):
dd52537da08049a82901b5b0c2c97b088e2e0032fbd5fb8118acb9abd4e2698d
features.rs (worker1):
c096e53f4a5cdfacf921f4b41b2bd7d3e94e128fb180ccc4fe88887c0274d883
loop_paths.rs (worker1):
b0abb61f174b765a727765bff005465de28e665164c10e2225d6c11aeb19fe5a
loop_paths/small_perimeter.rs (worker1):
5479f31754836abd71daa31fa9b399052613edcf92a2ff9f80e8500460e59a2c
```

After that caller rewrite, ordinary positive small-loop selections mask the
local shortcut. The remaining candidate is disagreement between the scaled
caller predicate and the local mm predicate at a floating-point boundary.
The following JavaScript binary64 expression check demonstrates possible
predicate disagreement; it is **not Native output or a reachable E2E RED**:

```javascript
const length = 40001096;
const scale = 0.000001;
const threshold = 6.3663721574934415;
length <= (threshold / scale) * 2 * Math.PI; // false
length * scale <= threshold * 2 * Math.PI;  // true
```

The limits are respectively `40001095.99999999` scaled units and `40.001096`
mm; converted length is `40.001096` mm. An axis-aligned square with scaled
side `10000274` has that scalar perimeter, but actual slicing geometry,
materialization, summation and exported config must establish reachability.
Do not label that constructed scalar as a measured sliced loop.

If a real case reaches this direction of disagreement, worker1's caller
supplies no override, but the local shortcut substitutes small speed instead
of the upstream default scarf cap. The reverse disagreement is masked by a
positive caller override. Other geometry/3D length differences have not been
validated and are not added to this slice.

## Non-negotiable decisions and proposed change

1. The complete-loop caller is the sole owner of small-perimeter selection.
   Default scarf speed only resolves the role-relative scarf setting and cap.
2. After approval and worker1 merge, remove the local threshold early return
   and remove the now-unused `loop_length` argument from `scarf_speed`.
   Keep `build`'s length calculation: slope geometry still needs it.
3. Preserve positive caller speed bypassing the scarf cap. Do not replace
   it with `min(small_speed, scarf_speed)` or reinterpret units elsewhere.
4. Do not change `features.rs`, `materialized`, `path` or `loop_paths` while
   worker1 owns them. The planned production edit is confined to `scarf.rs`.
5. Leave default slope ramp/raft behavior, zero role-speed handling, geometry,
   clipping, E precision, cooling, flow and formatter semantics outside this
   slice. Any independent discrepancy stays visible in the byte comparison.

Included: the `GCode.cpp:6515–6526` default scarf speed boundary and removal
of duplicate small-loop selection there. Deferred: loop speed ownership and
transport (worker1), independent ramp/raft issues, conditional scarf geometry,
mixed-role scarf geometry and broader speed-pipeline work.

## RED, GREEN and review gates

After both spec approval and worker1 integration, fetch/rebase latest
`origin/main`; first verify that merged caller behavior matches this analysis.

1. Build a real public byte-input classic scarf case and freeze its exported
   input/config. Establish a sliced complete-loop length and a threshold that
   makes the caller reject small-loop selection while the mm shortcut accepts
   it. Use distinct small speed and scarf default speed, with the relevant
   non-first layer outside ramping and without cooling/volumetric masking.
   Parameter selection must be recorded before Native comparison, not tuned
   to hide unrelated output differences.
2. Before production edits, add an E2E using the existing Native parity runner:
   fresh Native and Ares outputs, complete bytes, normalization only of the
   `; generated by` identity line. Preserve M73, statistics and footer. Freeze
   executable/resources/input/output identities and raw artifacts with the
   repeatable command. A feedrate witness supplements the complete assertion.
3. Commit RED separately. If no reachable case proves this shortcut's symptom,
   report that limit to the coordinator and do not implement GREEN. A scalar
   probe, private emitter test or unrelated earlier difference is insufficient
   evidence that this candidate is a real regression.
4. Implement only the approved removal, then require the exact same complete
   Native comparison to pass. Retain ordinary small-loop and non-small scarf
   controls, including absolute/percentage scarf speed and a cap above role
   speed; reuse suitable existing controls rather than add shallow unit tests.
5. Run `cargo fmt`, strict workspace/all-target/all-feature Clippy, workspace
   nextest and wasm32 check using independent `${TMP_ROOT}/ares-worker6-target`.
   Respect browser holds, one heavy task at a time and 16 test threads.
6. Run fresh same-round official and seeded parity control sets on candidate
   and base; candidate failure names must equal base or be a documented
   subset. Record identities, timeouts and exact artifacts; timeout is no verdict.
7. Submit the exact hash for review and wait for approval before any push.
   Rebase latest main, rerun required checks, then only fast-forward push.

This spec does not assert E2E divergence, test success or implementation
authorization. It establishes the upstream mismatch in ownership, the usual
masking after worker1, and the evidence required before a behavioral fix.
