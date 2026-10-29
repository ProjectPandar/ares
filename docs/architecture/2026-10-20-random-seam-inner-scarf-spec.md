# ARD: random seam position arithmetic exposed by inner-wall scarf

Status: approved SPEC (68037322e, amendment96a4bdec2: retain last-edge
guard). Pre-window preparation authorized on e81afae89 after the approved
stagger chain. Historical branch w5-random-scarf-spec was based on queued
6c3038b32 (main a5eb0f035); the preparation stack carries its unchanged
production scope as bf8834744. No FULL is authorized.

## Native-first, normally reachable witness

The two projects were actually exported by the official OrcaSlicer
2.4.2 CLI using `--export-3mf`, then sliced afresh from those exported
files. Both operations exited 0. Retained GUI settings:

- Scarf joint seam: Contour (`seam_slope_type = external`).
- Scarf joint for inner walls: OFF control, ON positive
  (`seam_slope_inner_walls = 0 / 1`; `PrintConfig.cpp:5628-5634`).
- Random seam; staggered inner seams enabled; three walls; Inner/Outer
  wall order; default scarf steps 10 and start height 0.
- Wipe disabled through the ordinary per-extruder `wipe = ["0"]`
  setting. This removes a separate wipe-retraction discrepancy, rather
  than filtering any G-code. This is a native-exported vector option,
  not an array supplied for a scalar option.

Exported control and positive differ in exactly ONE ZIP payload,
`Metadata/project_settings.config`, and exactly ONE JSON setting,
`seam_slope_inner_walls`. Geometry and other payloads are identical.

- Both official and seed-replay Native complete successfully.
- Control matches Ares's COMPLETE output in both modes, ignoring only
  the existing generator identity/date line.
- Positive has exactly one differing line in both modes:

      Native: G1 X114.679 Y118.918 Z18.329 E.02273
      Ares:   G1 X114.679 Y118.918 Z18.329 E.02274

  Raw line 20461 (20460 after removing the generator line).

Evidence root: `/tmp/w5-real-combos/`:
`scarf-{inner,ctl}-nowipe/exported.3mf`, `export.log`, `native.log`,
`seeded.log`, Native/Ares complete G-code;
`scarf-discovery-result.json` records fixture/ZIP/G-code hashes,
retained settings and exact differences. Recheck with the existing
Python executable:

    taskset -c 7,8 /nix/store/60m4rxhg2fldqaak400c0lry96ijrzqn-python3-3.13.13/bin/python3 /tmp/w5-real-combos/check-scarf.py

No repository fixture/test or production change is part of this SPEC.

## Read-only GDB trace: earliest observed unequal intermediate

CPU affinity was 7-8 throughout. Native is seed-replay binary
`1k0p5nlbs3zsfdcy08g3y5fb7bcm1qrv-...` with its matching DWARF debug
output; Ares is `/tmp/w5-targets/rs-red1/debug/ares`, commit
`6c3038b32`. No inferior function calls or memory/register writes.
Scripts/logs/JSONL captures are `scarf-{native,ares}-{target,random,...}`
under the evidence root.

The investigation followed writer delta backwards through scarf segment
geometry to random placement. For the target layer, matched by seed
bits and perimeter start index rather than parallel arrival order:

| Random selection input | Native AND Ares (identical bits) |
|---|---|
| slice_z | 18.299999237060547 |
| seed | (9.77500057220459, 9.77500057220459, 18.299999237060547) |
| selected index | 3 |
| selected position | (9.77500057220459, -9.774999618530273, 18.299999237060547) |
| edge | (0, 19.549999237060547, 0) |
| edge length | 19.549999237060547 |
| picked_len | 6.720310211181641, bits 1087835336 |

**The first observed unequal intermediate is final_seam_position:**

| Value | Native | Ares |
|---|---|---|
| final Y | -3.054689407348633 | -3.0546889305114746 |
| final Y f32 bits | 3225649160 | 3225649158 |

Final X and Z remain identical. Subsequent observations:

| Downstream value | Native | Ares |
|---|---|---|
| original inner-loop start Y, scaled | -2604688 | -2604685 |
| target scarf segment end X, scaled | 4679230 | 4679228 |
| cumulative scarf length, mm | 7.7613060000000003 | 7.7613050000000072 |
| total scarf length, mm | 12 | 12 |
| writer delta E | 0.022734997376042496 | 0.02273501590149438 |

Thus the writer receives values on opposite sides of 0.022735; formatting
is not the owning boundary. Native's writer call is from
`GCode.cpp:7039-7044` (`sloped->interpolate` and `dE * e_ratio`), but
altering scarf interpolation, flow or formatting would mask an earlier
geometry divergence.

## Exact upstream / Rust boundary

`OrcaSlicer/src/libslic3r/GCode/SeamPlacer.cpp`,
`SeamPlacerImpl::pick_random_seam_point`, final assignment at 1004-1005:

    final_seam_position = position + edge.normalized() * picked_len;

`edge` and `position` are `Vec3f`; `picked_len` is float. Eigen first
normalizes each vector component, then multiplies by picked_len, then
adds position in f32.

Rust destination:
`crates/ares-core/src/project_slice/seam_placement/alignment/random.rs::select`,
currently `position + edge.vector * (picked / edge.length)`.
That ratio-form scaffold changes operation order and is replaced only
at this cited boundary; it is not an alternative Ares seam algorithm.

## Included rewrite / non-negotiable decisions

1. Adopt `position + edge.vector.normalized() * picked` in f32.
2. Reuse existing `mesh::Vec3::normalized()` and `Mul<f32>`; no new
   normalize helper. The existing helper has Eigen's zero-vector guard.
3. Delete the obsolete `edge.length > 0.0` ratio guard with the ratio.
4. Preserve the existing forced last-edge acceptance guard. There is no
   evidence that it affects this witness. Upstream's possible float-
   remainder overrun is undefined behavior; replacing this guard with a
   Rust panic is not part of this arithmetic slice.
5. Keep random seed/hash, viable-list construction, lengths, comparison
   policy, defaults and option handling unchanged.

## Deferred / distinction from the earlier falsified hypothesis

The old bx-rand witness at slice_z12.5 had IDENTICAL random positions;
its E.0392/E.03921 difference was correctly located in staggered
interpolation and fixed separately. That witness CANNOT justify this
change. This new Native-exported inner-scarf witness is on slice_z18.3
and actually has unequal random final positions with identical inputs.

Do not change staggered interpolation, scarf splitting/detailing,
G-code flow, slope interpolation, formatting, wipe handling or random
constants. The wipe-enabled project had two additional retraction E
mismatches; those remain deferred and are not assumed to share a cause.

## RED → GREEN verification plan

- Freeze the ACTUAL Native-exported positive/control projects and ZIP
  manifests. Use merged `runner/native_api_cli.rs::NativeCase` for both
  fresh Native modes, public validated API and bare CLI. Like main's
  NativeCase users, Native availability is the only environment condition;
  no borrowed `ARES_PARITY_PROFILE_ENTRY_REFERENCE` gate or extra gate
  that can silently omit the cases from required-Native runs. Use the
  existing whole-stream comparator. RED must show
  control PASS and the exact single positive E-line FAIL in both modes.
- After the bounded change, read-only GDB must show the target random
  final Y bits equal Native (3225649160), then check downstream geometry
  and writer delta. Both complete-output tests must pass. These are
  requirements to verify, NOT a claim that the proposed fix is already
  sufficient; if any mismatch remains, stop and trace before expanding.
- No FULL during discovery/SPEC. After implementation and at the assigned
  merge slot, rebase to then-current main and obtain exact-hash official
  and seeded FULL plus rustfmt, clippy and wasm32 evidence.
