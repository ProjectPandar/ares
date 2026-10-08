# Stock ORCA_ERROR core-owned rows: family audit with live Ares replays

Date: 2026-10-08. Worker5. **Report only — no production change.**

## Scope and count reconciliation

List source: the stock baseline machine record
([2026-10-08-stock-1cf-baseline-record.json](2026-10-08-stock-1cf-baseline-record.json)),
rows with `fresh_status == "ORCA_ERROR"`: **54 rows**. The requested "57"
does not exist in the docs: the 1cf report's family table totals 54
(20+8+4+4+5+3+10). If three further rows were intended (e.g. specific
INPUT_REJECTED rows), they are an open delta for the coordinator to name;
this audit covers the authoritative 54.

## Method

1. Per-row evidence dirs survive under
   `/tmp/worker3-stock-1cf-fresh-artifacts/` (command.json, status.json,
   stdout.bin). Stage, exit code and Native stdout extracted for all 54.
2. **Slice-stage rows (31)**: the exported input 3mf survives in the
   native cache. Each was replayed through the current public Ares API
   (`ares slice`, in-process ares-core) and the behavior recorded.
3. **Export-stage rows (23)**: the export staging jsons are dropped on
   failure and no input artifact survives; Ares behavior is cited from the
   existing controls and the historical read-only audit
   ([2026-10-07-stock-error-core-readonly-audit.md](2026-10-07-stock-error-core-readonly-audit.md)),
   not replayed.
4. Upstream cites re-verified in the seed-replay 2.4.2 source where the
   overlay carries the file (marked ✓); otherwise quoted from the baseline
   report (marked q).

## Family results

### 1. Minimum line width — Slice exit 205 (native `-51`) — 8 rows

Rows: Creality CR-10 SE 0.2, Ender-3 0.2, Ender-3 Pro 0.2, Ender-3 S1
Plus 0.2, Ender-3 V3 KE 0.2, Ender-3 V3 SE 0.2, LONGER LK10 0.2, LK10
Plus 0.2.

- Upstream: `Print.cpp` width validation — `err_msg = L("Too small line
  width")` at **Print.cpp:1549** and **:1687** ✓ (report range
  :1524-1556/:1654-1671).
- Native: exit 205, stdout `run found error, return -51, exit...`,
  stderr `Too small line width`.
- **Ares (live)**: exit 205, identical stdout framing, stderr
  `Too small line width`. **Byte-level exit/output parity.**

### 2. Empty plate / no suitable object — Slice exit 206 (native `-50`) — 4 rows

Rows: Anycubic Kobra 3 0.2/0.4/0.6/0.8.

- Upstream: scene/build-volume eligibility — Model.cpp:692-702 /
  :3423-3469, BuildVolume.hpp:91-106, OrcaSlicer.cpp:5710-5724 (q).
- Native: exit 206, stdout `run found error, return -50, exit...`.
- **Ares (live)**: typed rejection
  `InvalidInput("One of the plate is empty or has no object fully inside
  it. Please check that the 3mf contains no empty plate …")`. Semantic
  match; **presentation deviation at the measured layer**. Layers:
  the core API rejects through the typed boundary (measured); the CLI
  error-family exit-code port is in main (`f68959e33`, merged with
  1a8c92f55's retained-exit rework), but the live replay of these four
  rows still surfaces the typed error rather than `-50` framing —
  classifying this detection point into the `-50` family is a worker4
  CLI-layer mapping question, not a core behavior gap.

### 3. Embedded post_process — Slice exit 237 (native `-19`) — 4 rows

Rows: Prusa MK3S 0.25/0.4/0.6/0.8.

- Upstream: CLI embedded post_process rejection — OrcaSlicer.cpp:1682-1690
  /:131 (q).
- Native: exit 237, stdout `run found error, return -19, exit...`.
- **Ares (live)**: identical stdout framing `run found error, return -19,
  exit...` with exit 237 (the `project_slice::validated::
  validate_post_process` control rejects even `[""]`). **Framing parity.**

### 4. Invalid custom G-code template — Slice exit 156 (native `-100`) — 5 rows

Rows: FLSun QQ-S Pro 0.4, Ginger G1 1.2/3.0/5.0/8.0.

- Upstream: custom G-code generation failure — **GCode.cpp:3644**
  `Failed to generate G-code for invalid custom G-code.` ✓ (report range
  :3636-3649; PlaceholderParser.cpp:918-932/988-1000/1726-1770 q).
- Native: exit 156, stdout `run found error, return -100, exit...`.
- **Ares (live)**: typed rejection
  `InvalidInput("Failed to generate G-code for invalid custom G-code.\n\n
  machine_start…")` — the existing O7 placeholder error aggregation, with
  the offending template name. Semantic match; **presentation deviation**
  (no `-100` exit framing). Closing owner: worker4's `-100` slice (the
  error-family exit-code port, `f68959e33`, in main).

### 5. Multi-head selector shape — Slice SIGSEGV (native crash) — 10 rows

Rows: Custom MyToolChanger 0.4, Flashforge Creator 5 0.6, Creator 5 Pro
0.4/0.8, Prusa XL 5T 0.6, Snapmaker U1 0.2/0.4, WonderMaker ZR Ultra
0.2/0.4/0.6.

- Upstream: the source-bounded historical unchecked selector path
  (baseline report; no crash-emulation contract exists).
- Native: SIGSEGV, empty stdout/stderr (status code null).
- **Ares (live): sliced OK** — produced complete G-code. This is the
  known native-crash-vs-ares-success deviation; per the baseline report
  the handoff is **worker1 diagnostic-only** (exact call proof), not a
  numerical parity or crash-emulation candidate.

### 6. Loaded-config validation — Export exit 251 (native `-18`/`-5`) — 20 rows

Rows: Anycubic Kobra S1 Max 0.25/0.4/0.6/0.8 (`retraction_distances_when_cut=0`
below the 10-minimum range), Kobra X 0.4, Creality K2 0.4, SPARKX i7
0.2/0.4/0.6/0.8, RH3D E3NG v1.2S 0.2-0.6, SeeMeCNC BOSSdelta 500
0.4/0.5/0.7/1.0 (height 2100 above 1000), Snapmaker U1 (0.4+0.6).

- Upstream: loaded-config validation — PrintConfig.cpp:10391-10394
  (✓ verified: per-key `error_message.emplace` shape at :10391-10394),
  :10504-10559, OrcaSlicer.cpp:3647-3653 (q).
- Native: Export-stage failure, stdout `run found error, return -5/-18,
  exit...` with the aggregated range/firmware text.
- **Ares behavior (static, not replayed)**: the existing options
  loaded-config validation matches 19/20 historically; Snapmaker U1
  0.4+0.6 remains the single open exact-case observation (historical
  audit). Ownership: worker1 exact-stock confirmations/range slices.

### 7. Empty-vector `set_at` exception — Export SIGABRT — 3 rows

Rows: Cubicon xCeler-I/Mini/Plus 0.4.

- Upstream: `ConfigOptionVector::set_at(): Assigning from an empty
  vector` — Config.hpp:438-446, OrcaSlicer.cpp:3290-3302 (q; the string
  is not carried in this source overlay).
- Native: Export SIGABRT.
- **Ares behavior (static)**: `profile_input/overlay::filament_slot`
  structured error instead of abort — the **approved error-vs-abort
  presentation deviation**; no production candidate.

## Summary

| Family | Rows | Native | Ares (live where replayable) | Classification |
|---|---:|---|---|---|
| min line width | 8 | exit 205 `-51` | exit 205, same framing+text | **parity** |
| empty plate | 4 | exit 206 `-50` | typed `InvalidInput` same semantics | semantic match, presentation deviation |
| post_process | 4 | exit 237 `-19` | same framing, same exit | **framing parity** |
| invalid template | 5 | exit 156 `-100` | typed `InvalidInput` with template name | semantic match, presentation deviation |
| multi-head selector | 10 | SIGSEGV | sliced OK | native-crash/ares-success (worker1 diagnostic handoff) |
| loaded-config | 20 | Export failure | static controls, 19/20 historical match | worker1 exact-case scope |
| empty-vector abort | 3 | Export SIGABRT | structured error | approved deviation |

No missing Ares detector, no valid-input divergence and no production
change is established by this audit. The 31 Slice-stage behaviors are
live public-API measurements against the surviving 1cf artifacts; the 23
Export-stage rows remain static-control citations until their inputs can
be reconstructed.
