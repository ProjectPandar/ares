# Legacy brim_ears boolean versus active brim_type

## Claim and upstream boundary

Bounded option-coverage evidence for the standalone `brim_ears` boolean.
Do not conflate this key with the `brim_type = brim_ears` enum value.

Source owners:
- `libslic3r/PrintConfig.cpp:1751-1757`: standalone coBool definition/default.
- `PrintConfig.cpp:427,1697`: distinct `brim_type` enum token `brim_ears`.
- `libslic3r/Brim.cpp:310-354,455-460,536-554`: ear geometry is selected by
  `object->config().brim_type == btEar`, not the standalone boolean.
- `libslic3r/Config.cpp` option deserialization/export owns preservation of
  recognized configuration values; exact functions will be cited in admission.

Rust destination boundaries already exist in `ares-core/src/options.rs`
(`brim_options`), `options/brim` parsing and `brims/ears.rs`. These remain
compatibility shells around upstream Brim concepts, not an Ares-owned adhesion
pipeline. This slice initially changes only CLI E2E/sweep evidence.

## Included six-input matrix

Use a fixed single cube10, one PLA, single physical nozzle, no support/tower,
first-layer width/height fixed and positive brim width/gap held constant.
Three selectors × standalone bool0/1:
- `brim_type = no_brim`: two inactive controls.
- `brim_type = outer_only`: two outer-brim controls.
- `brim_type = brim_ears`: two actual ear-selector cases, with default angle125
  and detection length1 fixed.

Freeze exact valid enum spelling and coherent settings from Native admission,
not assumptions in fixture construction. Verify all ZIP members/settings except
selector and standalone bool are unchanged. Existing generic `brim_type`, angle
and detection-length coverage is retained, not declared globally absent.

Expected source fact: changing the standalone boolean does not select ear
geometry. Prove this on each selector with Native outputs and report any
contradiction before writing acceptance tests. Whole output can still differ
in CONFIG_BLOCK; do not assert byte equality across bool inputs or filter the
main comparator. Ear-selector versus no-brim must show real first-layer brim
motion; outer-only versus ears should distinguish actual contour behavior.
If the simple cube fails to distinguish geometry, revise the spec/input
rather than weakening a witness or silently changing the input.

## Acceptance and deferred behavior

Native first, then six test-first CLI E2Es, each complete validated API and real
CLI output versus live Native using unchanged strict comparator and existing
wall-clock exemptions. Add value-bearing authored boolean plans with bounded
sweep dispatch. Direct GREEN is valid.

Defer painted/manual ear positions, holes, multiple objects, complex contour
simplification, angle/detection grids and all production geometry changes.
Any mismatch stops for a source-attributed operation-level report and a
separate production slice. No fallback, guard, tolerance, disabled assertion
or new adhesion scheduler.
