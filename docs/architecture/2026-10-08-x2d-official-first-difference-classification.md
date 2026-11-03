# X2D official retained-output classification

Read-only worker6 audit. No slicing, builds, heavy tasks, production edits or
oracle retries. Source revision: OrcaSlicer
`8500fcdccaa10b5099ac20d252af3a7c560046f1`.

## Scope and verdict

Worker3's `2026-10-08-timelapse-retained-output-verification.md` identifies
three base Native timeouts and two candidate whole-file failures. All three
test names use the same input SHA256:
`22d3813ddbbfbc40bd3955fb35d5f6d19b7d8ee23472e7aa2e354e50a23cfe82`.
This is `tests/parity/validation/multi-filament-bridge-anchor-narrowed.3mf`.
They are not three distinct geometry/config cases.

Both whole-file failures are **UB_ADJACENT / non-PASS**, specifically an
uninitialized `PrintObject::m_id` read. They are **not** the
O8-X2D-SCALAR singleton-nozzle selector UB. No independent deterministic Ares
byte difference remains in these two captured assertion streams. This does
not promote either official run to PASS or prove general X2D parity.

## Per-test classification

| Test | Base | Candidate first difference | Conclusion and suggested owner |
|---|---|---|---|
| `bridge_over_infill_filament::bridge_over_infill_multi_filament_x2d` | Native Slice timeout, ~301.47s; no paired Ares output | Raw G-code line **1097**; generator-line-removed assertion line **1096**, byte offset **57060**. Ares `; printing object ksr_fdmtest_v4.drc id:0 copy 0`; Native `; printing object ksr_fdmtest_v4.drc id:9223372037920129024 copy 0` | Uninitialized object-id UB, no source-defined giant numeric ID to port. Coordinator/reference-build owner; keep emitter unchanged. |
| `narrowed_x2d_whole_file::narrowed_x2d_whole_file` | Native Slice timeout, ~300.97s; no paired Ares output | Raw and assertion line **1097**, byte offset **57075**. Ares same `id:0`; Native `id:13700763839274728836` | Same object-id UB boundary. Coordinator/reference-build owner; no independent SeamPlacer/flush/emitter fix supported. |
| `change_filament_flush_blocks::change_filament_flush_blocks` | Native Slice timeout, ~301.27s | Candidate nextest PASS, ~207.50s, but this test compares **only extracted change-filament blocks**, not the whole file. It does not persist its Ares output. Its Native file's first object marker is raw line **1097**, `id:9223372037920129024`. No whole-file first difference is asserted by this test. | Narrow block equality is observed; whole-file PASS is not established. Same input and source UB apply. Keep narrow test verdict distinct from official full-file disposition; coordinator/reference-build owner. |

The bridge assertion removes the generated-by line with `split_inclusive` and
preserves all other bytes. The narrowed assertion replaces the banner, then
uses `lines().join("\n")`; its printed arrays therefore retain a banner line
and omit the final newline. Line numbers above distinguish those conventions.
No comparator was changed during this audit.

## Source trace

All upstream paths below are under `OrcaSlicer/src/libslic3r/`:

- `Print.hpp:582`: `size_t m_id;` has no member initializer.
- `Print.hpp:468–469`: `PrintObject::get_id` directly returns it; `set_id` is
  the setter. This is separate from ModelObject unique/loaded label IDs.
- `PrintObject.cpp:85–116`, `PrintObject::PrintObject`: initializes base,
  transform and preview cache, but never assigns `m_id`.
- `GCode.cpp:2697–2698`: calls `GCode::set_object_info` only when
  `m_enable_exclude_object` is true.
- `GCode.cpp:8074–8078,8094–8096`, `GCode::set_object_info`: also returns early
  for BBL printers; the sequential `object->set_id(object_id++)` is later.
  A source-wide search finds this as the only `PrintObject::set_id` call.
- `GCode.cpp:5348–5351,5471–5475`: printing and stop-printing object comments
  read `get_id()` whenever `gcode_label_objects` is enabled.

The captured config footer has `exclude_object=0`, `gcode_label_objects=1`.
Thus comment emission reaches the uninitialized field without executing the
assignment path. Changing arbitrary 64-bit IDs across processes corroborates
the static source evidence; randomness alone is not the proof of UB.

Existing reference repair:
`scripts/orca-replay/object-id-initialization.patch` changes exactly that field
to `size_t m_id{ 0 };`. The retained seeded source
`/nix/store/g3wj93lyhjsvc32fmgvw0cdbd6191q55-orca-slicer-2.4.2-seed-replay-source/src/libslic3r/Print.hpp:582`
contains this initializer. `docs/architecture/ard-0068-seeded-slicer-context.md`
distinguishes authorized initialization repairs from official-binary evidence.
Rust `project_slice/gcode_emit/object.rs:55–66` already uses defined zero when
IDs are not assigned. No UB emulation or numeric fallback is warranted.

## Why this is not O8-X2D-SCALAR

The saved input's `Metadata/project_settings.config` contains:

```text
nozzle_diameter = [0.4, 0.4]
nozzle_volume_type = [Standard, Standard]
extruder_type = [Direct Drive, Bowden]
filament_colour = [#26A69A, #26A69A]
flush_volumes_matrix = [0,280,280,0,0,280,280,0]
exclude_object = 0
```

O8's documented `OrcaSlicer.cpp:3457–3463` failure needs a missing per-nozzle
selector (one type entry for two nozzles). That prerequisite is absent here.
Both stored selector vectors have two entries and the matrix has eight cells.
No unsupported-NozzleVolumeType message was found in the retained narrowed
Native log. This excludes the **specific recorded scalar prerequisite**;
it is not an exhaustive absence-of-UB claim for the entire Native process.

## Exact retained evidence and comparison coverage

Official producer command records name `/tmp/orca-wrapper-inherit-cpu.sh`,
which delegates to `/tmp/orca-wrapper-fixed.sh`, then CPU1
`${TMP_ROOT}/ares-pi-arachne/scripts/orca-parity.sh`, with
`ORCA_APPDIR=/tmp/orca-official-242/squashfs-root`.
All three candidate command status records report exit0. No fresh execution
was performed. The record establishes the historical wrapper/environment;
it does not newly attest the current executable's historical identity.

- Bridge candidate: `/tmp/bridge-multi-cKDamL/`, command `command-eCZf1j`.
  Native raw SHA256
  `3c4a5c62f732370e6f059ce80cf93d62d794374c05e07d443969eb8b35eb99cd`,
  6,412,734 bytes. Ares raw SHA256
  `520b3da57f8276bad8488cf73150512ee7b0a608bfb88bdeda2673879df9c938`,
  6,387,070 bytes. The assertion-array Ares SHA256 is
  `331827159281658d6baac34427836ee19fe89242c123fcb32ae3757ccd4ed79d`,
  binding this directory to the candidate panic rather than a later replay.
- Narrowed candidate: `/tmp/narrowed-x2d-whole-file-NLkeX5/`, command
  `command-s37LfD`; path appears directly in the candidate panic.
  Native raw SHA256
  `68324991e5ef1b2d3fdbd035038ea2c9b74e58d727e363baa934b3ce94cdc627`,
  6,414,158 bytes. Its exact Ares assertion-array SHA256 is
  `ff469e046fbd6f49e361bed645f184e5daed08366a9ce02d0f48a0dc26574543`;
  Native array SHA256
  `a6dc489d5d2bc900f2c7a2468f24723ea3d3133d0165507813af9b840237c126`.
  Ares bytes are recoverable from the retained panic, not an invented rerun.
- Flush-block candidate: `/tmp/change-filament-blocks-tigy8m/`, command
  `command-gzNkhr`; Native raw SHA256
  `83b09ca5e3c43adc3d51f54aca2644a5a098344867a18decbf7b674518988449`,
  6,412,734 bytes. Its recorded time overlaps the official candidate round;
  unlike the two failures, the successful log does not print this directory.
  Directory attribution is therefore weaker, based on time/input/producer;
  block-PASS evidence comes independently from the named nextest log entry.
- Base timeout directories from log: `/tmp/bridge-multi-BMWM3O/`,
  `/tmp/narrowed-x2d-whole-file-HyeDFk/`,
  `/tmp/change-filament-blocks-dhSekB/`. These remain INCONCLUSIVE.
- Authoritative test verdicts/arrays:
  `/tmp/worker3-timelapse-official-{base,candidate}.log`.

Decoded both candidate `left`/`right` byte arrays from the retained panic.
Each pair contains 279,768 split entries and **1,424 differing lines**.
Every differing line is a printing/stop-printing object comment, identical
except for the decimal `id` field; line counts and all other lines agree.
This is an exhaustive inventory of the captured assertion streams, not a
new comparator, ignored assertion or PASS promotion. Generator-only equality
remains false. The bridge raw streams also preserve the final newline and
show the same inventory. No hidden motion, E, M73 or footer residual was found.

Reproduction of the key read-only analysis requires no engines: parse the
decimal byte arrays following each named `thread '…'` panic in the candidate
log as Buffers; compare each byte and every newline-delimited entry. Extract
the saved config using `unzip -p <input.3mf> Metadata/project_settings.config`.
Use `sha256sum` on the original raw files to verify the identities above.

Suggested disposition: retain UB_ADJACENT/non-PASS for the two official
whole-file failures, name the object-id boundary explicitly, and keep the
O8 scalar boundary separate. Any future defined-oracle run must retain its
distinct initialized producer identity and full-byte assertions. No Ares
implementation slice is justified by this captured difference.
