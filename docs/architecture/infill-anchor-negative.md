# ARD: `infill_anchor < 0` admission — upstream negative-anchor domain

Scope B of the `anchor_projection.rs` admission-defect class. Follow-on to
`infill-anchor-dont-connect.md`.

## Problem statement

Ares' admission guard in
`crates/ares-core/src/project_slice/prepare_infill/bridge_over_infill/transaction/anchor_projection.rs`
rejects `infill_anchor < 0` (`anchor < 0.0` conjunct →
`UnsupportedProjectFeature("infill_anchor")`). Upstream has **no lower bound**
on this option: `infill_anchor` is `coFloatOrPercent` with `mm or %` semantics,
`max_literal = 1000`, and no `min`/`min_literal`
(`PrintConfig.cpp:3137-3160`, `PrintConfig.hpp:1203`). A negative value is a
schema-legal input that the upstream release binary consumes.

## Upstream evidence (OrcaSlicer 2.4.2, `src/libslic3r/`)

| Site | Lines | Content |
|---|---|---|
| `PrintConfig.cpp` | 3137–3160 | `infill_anchor` `coFloatOrPercent`, no `min`, `max_literal=1000`; enum values are all ≥ 0 but the type accepts any float |
| `Fill.cpp` | 978–989 | `params.anchor_length = min(anchor, anchor_max)` after percent projection — no sign check; a negative `infill_anchor` flows through as `params.anchor_length < 0` |
| `FillBase.cpp` | 1584–1585, 2251–2252 | `connect_infill` asserts `anchor_length >= 0.` — **debug-only**; the shipped release binary does not evaluate them |
| `FillBase.hpp` | 52 | `dont_connect()` (`anchor_length_max < 0.05`) bypasses `connect_infill` entirely, making negative anchors unconditionally reachable in release |
| `FillRectilinear.cpp` / `FillBase.cpp` | 3366–3373 / 1820–1828 | `chain_or_connect_infill` dispatch already ported |

Release-oracle semantics for `anchor_length < 0` are measured, not assumed:
the Native CLI decides what a negative anchor produces; Ares must match it.

## Rust boundary

`anchor_projection.rs` (~line 110–111): remove only the `anchor < 0.0`
conjunct, keep `!anchor.is_finite()` (NaN/Inf is not a sanctioned input —
upstream `strtod`/JSON path cannot produce it through the 3MF/preset channel;
same reachability note as `infill-anchor-dont-connect.md` §2).
`projected_anchor_lengths` (`sparse_anchoring/projected.rs:11-18`) already
computes `min(anchor, anchor_max)` exactly like `Fill.cpp:988` — a negative
value propagates to the ported connect backends with no further change.

## Included / deferred

- Included: any `infill_anchor < 0` (absolute or negative-percent) across the
  already-supported rectilinear-family sparse patterns.
- Deferred: patterns under the `sparse_infill_pattern` guard; the
  `calculate_infill_rotation_angle` template interpreter (`Fill.cpp:52-190`,
  spec slice C); `sparse_infill_density = 0` (candidate A, deferred).

## Verification plan

RED module `orca_parity/infill_anchor_negative.rs` (extends the
`infill_anchor_dont_connect` harness): Native-backed official + seeded,
fresh cache/root per leg. Contexts:

- C0 `infill_anchor=-1`, `infill_anchor_max=1000` (connect path taken,
  upstream release behavior = oracle),
- C1 `infill_anchor=-1`, `infill_anchor_max=0` (dont_connect path — negative
  anchor is trivially unreachable upstream; still admitted),
- C2 `infill_anchor=-50%`, `infill_anchor_max=1000` (percent projection of a
  negative value, `Fill.cpp:984-985`).

`sparse_infill_density=20%`, `fill_multiline=1`, rectilinear pattern,
`tests/parity/overhang.stl` (SHA256 pinned as in dont_connect). RED witness:
upstream produces infill G-code; Ares rejects with
`UnsupportedProjectFeature("infill_anchor")`.

## Non-goals

- No behavioral definition for what a negative anchor *does* — the Native
  oracle defines it; this slice admits the input only.
- No change to `!anchor.is_finite()` / `!anchor_max.is_finite()` or the
  `sparse_infill_pattern` / density guards.
