# Signed hot-plate range plan

1. Probe negative one and the signed integer minimum through the existing
   official export/full-output seam; inspect acceptance and temperature use.
2. Derive actual bounds from upstream numeric storage and validation, replace
   fixed interior probes with deterministic random generation where justified.
3. Fix only reproduced output discrepancies; preserve complete paired outputs.
4. Run workspace Nextest, fmt, Clippy and independent review, then commit/push.

See [spec](../specs/2026-09-30-hot-plate-signed-range.md) and
[ARD](../../architecture/ard-0035-hot-plate-signed-range.md).

## Evidence

- Official first-filament cases accept -1 and -2147483648 and emit those
  signed temperatures. `Config.hpp:995` stores `ConfigOptionVector<int>`;
  its representable lower limit is stricter than default schema `-FLT_MAX`.
- `/tmp/ares-hot-signed-probe.log` is an exploratory assertion failure:
  highest-temperature formula emits zero for a negative input, as
  `GCode.cpp:4010-4018` specifies. It is not an input rejection.
- `/tmp/ares-hot-signed-formulas.log` then exposes a real full-output
  mismatch: official vendor startup emits S0, Ares emits S-1. Ares maximum
  selection missed the zero initializer. Both effective formula and value
  are checked in the exported project.
- `/tmp/ares-hot-signed-auto-cache-red.log` exposes automatic startup's
  extra M190 S0. `GCodeWriter.hpp:25` initializes zero/reached, and
  `GCodeWriter.cpp:168-175` suppresses this unchanged command. The earlier
  auto-red run had an incorrect M190 assertion; it was corrected from
  official output before testing Ares. `/tmp/ares-hot-signed-fixed.log`
  accidentally ran the old binary during compilation and is not green evidence.
- `/tmp/ares-hot-signed-final.log`: 24 complete paired outputs pass after
  compilation, covering min/max/seeded/-1/0/55 across both formulas and
  vendor/automatic startup. Full artifacts are retained beside the log;
  the seeded value is -691504564. Tests verify heating commands, including
  absence of automatic zero-temperature waits, before strict byte comparison.
- `/tmp/ares-hot-signed-options.log`: all 11 complete outputs pass for
  hot-plate range, all six plate types and both formulas. Completion gate
  exits zero. Historical matrix updates only hot-plate status: 615/650;
  case/compared/rejected counts do not change. This is not a fresh full sweep.
- `/tmp/ares-hot-signed-printer-replay.log`: 973/973 prior successful printer
  projects reproduce their cached full official outputs with the repaired
  implementation. Input inventory is `/tmp/ares-nozzle-printer-replay-input.H1GVyv`;
  new complete pairs are in `/tmp/ares-hot-signed-printer-replay`. This is cached
  regression evidence, not a fresh official full-printer/default sweep.
- `cargo clippy`, fmt and diff checks pass (227 existing core warnings).
  `cargo check` passes for core+WASM adapter on wasm32-unknown-unknown and
  core on x86_64-pc-windows-gnu / aarch64-apple-darwin; logs are
  `/tmp/ares-hot-signed-{clippy,wasm,windows,macos}.log`. These are compile
  checks, not runtime certification of those platforms. Changed files stay
  below 400 lines; no comparator changes or debug instrumentation remain.
  Checks include pre-existing geometry changes, excluded from this commit.
- `cargo nextest run --workspace` exits zero: 7369 passed, 9 skipped,
  262.830 seconds, `/tmp/ares-hot-signed-workspace.log`. Environment-gated
  offline passes are not counted as fresh official comparisons; the 35
  explicit fresh output pairs above provide that separate evidence.
- Independent read-only six-aspect final review approves with no repair
  items, after verifying all 35 fresh pairs, cached replay summary, effective
  values/formulas/startup modes and terminal checks. Overall full Option and
  all-printer/default parity remains incomplete.
