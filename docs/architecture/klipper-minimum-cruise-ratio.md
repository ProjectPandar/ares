# Klipper acceleration-to-deceleration suffix rewrite

Scoped implementation plan; runtime RED precedes production edits.

## Non-negotiable source boundary

Port `libslic3r/GCodeWriter.cpp::append_klipper_accel_to_decel:475-481` and its
callers `set_acceleration_internal:428-471` / `set_accel_and_jerk:535-583` from
OrcaSlicer **345efec64c7b95a05ce5cf96df0dde66a158835c**. Upstream source SHA256:
`fe82d6b749c17d38b7595a8d46bb0b3a46eec338ebdce06faea1bcd5b9e137e6`.
The rewrite destination is
`crates/ares-core/src/project_slice/gcode_emit/motion/acceleration.rs` and
`crates/ares-core/src/gcode_writer/acceleration.rs`, not a new pipeline.
Existing motion emission is a temporary compatibility shell around the upstream
writer; replace its incomplete suffix with the cited helper semantics.

When accel-to-decel is enabled and acceleration is actually emitted:

1. Append literal ` ACCEL_TO_DECEL=` then acceleration * factor / 100.
2. Append literal ` MINIMUM_CRUISE_RATIO=` then 1 - factor / 100.
3. Format both through `float_to_string_decimal_point`'s default six-significant-
   digit decimal-point semantics (`LocalesUtils.cpp:110-145`), not integer casts
   or arbitrary precision. Preserve arithmetic evaluation order.
4. Retain command spacing, parameter order, comments, LF, flavor selection,
   acceleration clamping, unchanged/zero suppression, and jerk merge ordering.
   Disabled accel-to-decel emits neither suffix. Jerk-only updates do not
   synthesize an acceleration suffix.

Include both actual motion-emission callers and the existing writer acceleration
shell so they cannot disagree. Share the literal helper as
`GCodeWriter::append_klipper_accel_to_decel` across those three call sites and
reuse the existing crate-level `options::format_number` general-six-significant-
digit formatter for both values. Do not expose private motion modules, copy a
formatter, or introduce a new numeric framework. Existing motion jerk formatting
and all non-Klipper commands remain unchanged.

## Public RED witnesses before implementation

Run these six existing tests without changing their input-generation behavior.
The four fixed fixtures retain their exact SHA256s below. The two sync tests
produce fresh 3MF exports by design: the saved routing hashes identify earlier
containers, not immutable fixtures. Fresh RED actually replayed
`f93d4639c4e2edf3653e3d715b2b43a06670c276ba55917abdd7335f5df442c8`
(without sync) and
`a38066c1934fb31177cba401efd44fb6d5e4e4795f9ef461fda810a8a9f87abf`
(zero delay). Record the actual exported SHA for each run, do not edit the tests
to recreate historical metadata. All six must still demonstrate the specified
missing-suffix first difference.

| Existing public test | Original input SHA256 | Native ratio |
|---|---|---:|
| `case_support::sequential_origin_label_flush::sequential_origin_label_target` | `baa6c8d352d5e62c400cc450d50de643ce868fcc619a71352748f89aabe7aab1` | 0 |
| `case_support::support_base_extension_priority::support_base_stock_control` | `b3f72543f8794281e1d99ef784a993dbb0523bfbe96468403cec4b21da34a5a2` | 0 |
| `object_emission_cases::fuzzy_allwalls_rng_order::fuzzy_stock_native_control` | `5d4a5f95fd4c12244e87b17e9e64052f56d66fd83df405438fb4c4e34c8c524c` | 0 |
| `runner::native_jerk_semm_migration::e5max_klipper_jerk_switches_match_native` | `c503c92c8f524e16b4223d6349b572f333634bac418b7874236e64ceb536eaff` | 0.75 |
| `sync_time::sync_time_klipper_without_sync_matches_native` | `c1603fa39b1114e4a2efe85640610d9435be89e108e97c9b07ce8ea8ade131bd` | 0 |
| `sync_time::sync_time_klipper_zero_delay_matches_native` | `a48c9b379764a6ea0763c6fa75831f733577db85516865079b7b757e5b69b877` | 0 |

Saved observations are routing evidence, not a fresh test claim. Fresh public
Native/API/CLI execution must confirm original inputs and the actual first
complete-output difference. No private-state test, mocked oracle, regenerated
input, role-only comparator or filtered M73/Klipper record may replace it.

The upstream literal witnesses include:

```text
SET_VELOCITY_LIMIT ACCEL=1000 ACCEL_TO_DECEL=1000 MINIMUM_CRUISE_RATIO=0
SET_VELOCITY_LIMIT ACCEL=2000 ACCEL_TO_DECEL=500 MINIMUM_CRUISE_RATIO=0.75
```

These are cited command examples, not committed Native output goldens. All
runtime Native outputs, replay records, original-input hashes and comparison
artifacts stay external. Preserve the original test names and whole-output
assertions; do not modify ROOT registrations or weaken failure policies.

The explicitly selected CLI family is:

```text
test(/accel_to_decel_precision|klipper_separate_jerk_precision|klipper_separate_jerk_xy_clamp|klipper_velocity_limit|merged_jerk_precision|merged_jerk_tolerance|velocity_limit_comments|adjacent_motion|initial_travel_uint|native_jerk_semm_migration|power_loss_recovery|sync_time|layer_fan_priority|object_emission_cases|print_validation::positive/)
```

Add the six exact cases above, the separately published gap-fill input witness
SHA256 `d8f74deff1274800b04f021ba812c5fb84c1629e45981653218963e2cc3a1e37`
for diagnostic confirmation only, and the existing core internal
`gcode_writer` test selection. At this base those internal tests were retired:
no new private tests or restored registrations are permitted, and no internal
coverage is claimed. The requested internal command was stopped during
compilation after retirement was explicitly confirmed. That diagnostic harness/input is not committed
in this slice; preserve its unfiltered earlier M73 difference and identify
suffix observations separately. Before/after inventories must match by name;
no missing selected tests or altered assertions may manufacture zero regressions.

## Verification, failure model and exit

- Freeze the six public RED verdicts and input identities before production
  edits, then execute the same cases after the literal port.
- A gcode-writer family before/after inventory must show **zero PASS-to-FAIL**;
  retain every prior failure separately. Family selection must be explicit,
  not a guessed broad parity sweep or a reduced passing subset.
- Failures to exclude: suffix omitted on either emission path, ratio derived
  from the wrong factor, unsigned truncation, wrong precision/decimal separator,
  altered comments/order, unexpected suffix when disabled or on jerk-only
  emission, state/cache or non-Klipper regression.
- Formatting and behavior claims require actual public/runtime evidence. If a
  repaired command exposes a later strict whole-output difference, report that
  residual without claiming full-case parity or masking it to reach GREEN.
- Fmt, workspace/all-target clippy with denied warnings, and precise diff checks
  must pass.

## Executed outcome

The fresh six-case run reproduced all six missing-suffix
first differences. The explicitly selected 122-test CLI family (including the
six and temporary seventh diagnostic) completed before and after the port:
**39 PASS / 83 FAIL / 0 ignored on both runs, zero PASS-to-FAIL**. No existing
whole-output assertion, test name, input-generation rule or comparator changed.

Across the six real after-exports, every ordered acceleration command matches
Native literally: respectively 851, 415, 415, 207, 173 and 173 records, **2,234
records total**. The expected ratio0/ratio0.75 initial commands are present.
Directly retained CLI acceleration records in the three byte-API/CLI witnesses
also match. This is emitter diagnostic confirmation from complete real exports,
not a role-only replacement for the strict test boundary.

**None of the six strict complete-output tests is GREEN.** Their new first
differences are M73 timing/placement: sequential-origin line110, stock support
line110, fuzzy stock line110, E5Max line90, sync-none line90 and sync-zero line85.
Those failures remain visible and belong to the M73 timing slice. The seventh
exact d8 input keeps its original line19 M73 P0 R8 versus R7 first difference;
its literal acceleration suffix now matches. No M73 line was filtered.

The temporary seventh harness/registration/input were removed after the family
comparison and are not part of the committed slice. Retired internal writer
tests were not restored. Do not interpret
this source-correct literal suffix port as full-case parity closure.

Deferred: GCodeProcessor M73 timing, FillBase gap insertion/order, geometry,
configuration semantics and unrelated writer features. Their earlier or later
complete-output differences remain independent slices, not justification
for changing this slice's comparison boundary.
