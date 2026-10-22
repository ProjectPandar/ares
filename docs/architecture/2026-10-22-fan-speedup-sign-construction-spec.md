# Signed FanMover construction handoff — source-owned SPEC

## Stage and owner

COORD accepted discovery and authorized docs-first SPEC → test-only RED on exact
main `ad5111f0b4135eaa2ad88ea78ac7230200808941`. This commit is SPEC only;
separate RED follows. No production code, GREEN, FULL or integration is authorized.

Pinned upstream: `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
This is a **libslic3r parent-to-FanMover constructor rewrite slice**, not a new
Ares cooling scheduler, fan timing policy or pipeline design:

- `src/libslic3r/GCode.cpp:3732–3740` (construction arguments3736–3738) and the
  duplicate path`:3830–3838` (arguments3834–3836) gate/create FanMover.
- `src/libslic3r/GCode/FanMover.hpp:63–70` owns the constructor signature:
  writer, float nb_seconds_delay, bool with_D_option, bool relative_e,
  bool only_overhangs, float kickstart; its positive-delay clamp/storage.
- `FanMover.cpp:319,336–368,495` consumes the delay in the existing buffer pass;
  those algorithms are adjacent behavior, not being independently redesigned.

## Exact included literal contract

Let t be the original `config.fan_speedup_time.value` (double), k be kickstart.
Both cited GCode.cpp call sites have the same explicit contract:

1. Preserve activation `t != 0 || k > 0`.
2. Pass delay **std::abs((float)t)**: convert to f32/float **first**, then take
   the float magnitude. Do not retain the sign or move abs to an f64 policy.
3. Pass **t > 0** as a separate `with_D_option` constructor argument, evaluated
   from the **original** value, not the absolute/rounded magnitude. Negative
   values are false; positive values true; either signed zero false. A positive
   value rounding to float0 still has true sign flag. No new numeric admission,
   saturation or tolerance contract is invented here.
4. Preserve relative_e and only_overhangs arguments. Kickstart retains the
   existing source float conversion/semantics; do not widen scope to its logic.
5. FanMover's own delay clamp remains literal: when supplied float d > 0,
   max(0.01f,d), otherwise0. Parent conversion and constructor clamp are distinct.

**The >0 flag is not optional just because the negative witness exposes abs.**
`FanMover.hpp:35,63,67` names, accepts and stores `with_D_option`. The pinned
source has no further read in FanMover.cpp/hpp. Preserve/document that exact
opaque constructor bit; do not infer an output D word, invent lead/lag semantics,
reinterpret it as only_overhangs or collapse it into the magnitude. Its currently
inert downstream behavior must stay inert. No implementation is written here.

## Rust destination / scaffold replacement boundary

`crates/ares-core/src/project_slice/gcode_emit/layers/fan_setup.rs:7–20::gate`
is the parent handoff: it preserves the nonzero gate but passes raw t to
`gcode_emit/fan_mover.rs::FanMover::new`. The latter currently takes signed f64
and computes positive-only float delay (`:90–96`), losing negative magnitude;
its current constructor shell has no explicit with_D_option argument/state.

The future rewrite must represent the cited parent float-magnitude **and original
>0 argument** contract across these two private boundaries. The existing Rust
FanMover is a temporary compatibility shell around the cited libslic3r class,
not an Ares-owned pipeline. Constructor signature/state details must follow the
upstream contract without inventing currently absent flag effects. Keep the
existing buffering, role classification, fan command interpretation and writer
handoffs unchanged. Pure platform-neutral core logic must remain usable on
browser WASM, Windows, macOS and Linux; no oracle/filesystem/UI/terminal in core.

Deferred: all other FanMover behavior (split interpolation, arc treatment,
kickstart state machine, slow-fan removal, role policies, custom G-code barriers,
flavor/PWM formatting), cooling/ironing/geometry, timer/M73 repair, public option
validation and flat SliceOptions compatibility. No global writer guard, fallback
or empirical numeric correction. An eventual residual means STOP/report, not
adjacent repairs. GREEN needs a later independent approval.

## Frozen two-input public E2E RED

Accepted discovery root:
`/home/indexyz/ares-evidence/w3-fan-speedup-sign-readonly-ad5111f0b/README.md`,
manifest `e168427bef6b48379d45684ef15debe2c7fd3d2fe4f9e26d4807c1bed8e0393d`.
Copy its final active/inputs-v2 input bytes, do not regenerate.

Origin is tracked `tests/parity/modifier-bottom-shell/classic-four.3mf`, SHA
`438bf4c61895a7a8d51e3c61a70fa8df120a9f3a607d63a4291d359984154a7c`.
The stored Bambu Standard-Mine profile is customized, not pristine factory.
Both inputs share fan_min_speed=[20],fan_max_speed=[20],fan_speedup_overhangs=0;
all other original settings, geometry, placement, modifier bottom-shell4 and
non-settings ZIP payloads remain exact. Target versus control changes only t:

| Input | t | SHA256 |
|---|---|---|
|control.3mf|0|17896c6fdf4d5d570c604843fafd049cbc1ac6cde86a21c07dd20ebfdd8a13b2|
|target.3mf|−3|19507db8da1597d32a348f299edaa8d24a5e16aa203b6f3150da6ca308dc049d|

GUI reachability: `GUI/Tab.cpp:4552–4556` Cooling Fan field;
`PrintConfig.cpp:3830–3845` advanced coFloat without min override;
`Config.hpp:2478` default min−FLT_MAX; `GUI/Field.cpp:392–443` numeric range.
Native slices and retains −3; the public validated API succeeds, not refusal.

Use exactly two direct calls to unchanged `runner::native_stream_case::check`,
last argument **(false, |_, _| {})**. Native double replay precedes public
validated API and actual CLI, then complete ordered equality/raw API==CLI.
Commit only input3MF/SHA records and thin test wiring; no Native output/golden,
new comparator/helper, private call-count assertions, skipped/ignored/inverted
checks, masks or tolerance. Time/M73/E/F/wipe/footer remain in the comparison.

Discovery observed control PASS/target complete inequality both modes, first
byte147712/line4964 Native M106 S255 vs Ares CHANGE_LAYER; diagnostic diff was
one fan command relocated66 lines. Those are discovery facts, **not expected
output constants** in tests. Initial100%-fan negative/failed ZIP preparation
remain discovery history, not extra formal cases or a way to waive failures.

## Exit and approval barrier

Commit this document alone, then separate test-only RED commit with the two
frozen inputs. On the exact clean RED tip run official and seeded directed2:
expect1 control PASS and1 real strict output FAIL per mode, stable successful
Native and successful raw-equal API/CLI. An oracle failure/rejection is not RED.
Target comparison may panic before wrapper-tail; do not claim that tail ran.
Run fmt and locked all-target Clippy, retain actual full logs/exits/provenance,
submit RR, **stop before any production code**. No FULL/rebase/merge authorized.
