# Type2 wipe-tower activation verification (TiQ2)

Status: **verification GREEN; docs-only REVIEW pending**. Owner: worker1;
worker2 owns G4 only. Approved verification SPEC: `1babb7ce2`.
No production change or new test is proposed. This closes the bounded
`21c18b79d` enabled-input verification obligation, not tower-active geometry
or all Type2 behavior. See [ARD-0105](ard-0105-smooth-type2-timelapse-port.md)
for the independently landed planning/emission boundary.

## Historical correction

The historical divergence report originated in pi-arachne's
bridge_over_infill multi-filament E2E. The closure in `2c489196a` was
withdrawn: its original ZIP contained `enable_prime_tower="0"`. The apparent
"1" was an edited extracted configuration that was never repacked into the
input used by the slicers. Disabled outputs did not establish enabled parity
or identify the suppressing gate. The speculative single-used-tool/front-layer
ToolOrdering explanation was unproven; the measurements below supersede it.

Original input:
`${TMP_ROOT}/ares-orca-cache/19f06bc6065b3213/6e02aea054799098-fae0cc8012f785c5.3mf`,
SHA256 `74ada1cbfe1e28ce7f52007bd4f77f80f3014488ee21fb580a27b7c3f172de3a`.

## Real enabled input and complete-output verification

The original ZIP was repacked at runtime, changing only the semantic value
`enable_prime_tower` from "0" to "1" in `Metadata/project_settings.config`.
The configuration serialization changed; every other ZIP member is byte-exact.
Reopening verified the new value, Type2, traditional timelapse,
`single_extruder_multi_material=0`, two filament diameters and two nozzles.
Enabled ZIP SHA256:
`5f26ac862bbef14762ff38a5f14fbed2903690ca80d117c7aa7febd5b3c1697e`.

Official Native ran first, then the public Ares CLI on those exact bytes.
Both exited0 and produced 33995 G-code lines, exactly equal after normalizing
only the one generated-by banner. Both contain zero `;TYPE:Prime tower`
sections. Seeded Native debugger runs on the same input also produced identical
complete output under that same banner-only comparison; debugger Ares runs
retained the original Ares G-code SHA.

Measured **actual emitted tool use**, on both sides:

- Two used tools: T0 and T1, with positive filament consumption 605.01 and
  380.79 mm respectively. Configured slot count is not the measurement.
- Seventy T commands, including initial/same-tool selection; 68 transitions
  between different tools. The independently serialized footer likewise reports
  `total filament change = 68`.
- First ToolOrdering layer: Z0.2, tools `[0]`, has_object=true,
  has_support=true, wipe_tower_partitions=1, has_wipe_tower=true.
  There are 75 ordering layers, with tools0/1 used overall.

Thus this is **not** a zero-toolchange or false-front-layer suppression case.

## Measured upstream reason and matching Rust gate

Source pin: `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

1. `PrintApply.cpp:1127–1132,1620–1621` passes the apply-time
   `Print::extruders(true)` count into `DynamicPrintConfig::normalize_fdm_2`.
   `Print.cpp:451–547` collects the then-existing object regions, model-volume
   extruders and support selections. The later generated ToolOrdering tool set
   is a different lifecycle observation, not a replacement for that count.
2. Native runtime probe recorded normalization `(num_objects,used_filaments)`
   `(0,0)` initially, then **(1,1)** at the late Print::apply call. At this point
   the region construction needed for the later two-tool output has not yet
   provided that later ToolOrdering inventory. Do not label the final print
   single-tool merely because the apply-time count was1.
3. `PrintConfig.cpp:8688–8706` sees traditional timelapse, no wrapping and
   used_filaments=1; `:8703–8706` clears the **effective** enable flag.
   The original/full configuration serialized in the footer still records
   `enable_prime_tower = 1`; it is not proof of the effective Print flag.
4. Native `Print::has_wipe_tower` (`Print.cpp:3290–3302`) was measured returning
   false with effective enable=0, wrapping=0, smooth=0, spiral=0 and two
   filament diameters. `Print.cpp:2422–2431` therefore uses the ordinary
   ToolOrdering path instead of `_make_wipe_tower`; the latter had zero measured
   breakpoint entries. `GCode.cpp:2774` also requires this Print enable gate.
   The `Print.cpp:3402–3404` early return / `ToolOrdering.hpp:236` front-layer
   predicate is **not** the cause: the measured front-layer flag is true.

Rust already ports that lifecycle in
`project/effective_config/phases.rs:78–99`, then
`options/project_fdm_normalization/stage2.rs:15–22`. Runtime Ares observations:
FirstPreRegion enable=true; FinalPostNormalize enable=false, traditional
and no wrapping in both. Its ToolOrdering gate (`gcode_emit/tool_ordering.rs:
292–297`) measured prime=false, first_extruder=Some(0), while the front layer
has the same `[0]` tools, partition1 and true tower flag as Native.
`gcode_emit/wipe_tower.rs:31–39::type2_requested` requires the effective enable
flag, so Ares suppresses the tower through the same earlier configuration gate,
not a guessed ToolOrdering suppression or coincidental omission.
Ares's apply-time count is inferred as1 from the observed true-to-false
transition and existing Traditional/by-layer/no-wrapping predicate; optimized
locals did not expose that count directly. Native's count was read directly.

Consequently `WipeTower2.cpp` planning/generation and
`GCode.cpp::WipeTowerIntegration` tower emission are not activated by this
witness. No change to either boundary is justified, and no new RED is invented.
The K3M normalization mechanism remains adjacent/deferred; this records only
which already-ported normalization predicate this TiQ2 input actually reached.

## Repeatable evidence, scope and limitations

All Native products remain outside Git at `${TMP_ROOT}/ares-type2-activation/`:

- `enabled.3mf`, `input-audit.json`: original/input/member hashes and reopened
  settings; unchanged non-config members verified.
- `official-command.json`, `official-native/execution.json`, raw channels,
  result JSON and complete G-code; `ares-command.json`, `ares-current/`:
  exact-input sequential public CLI execution.
- `native-normalize.gdb`, `native-normalize-state.json`,
  `seeded-native-normalize/`: Native apply counts/effective Print gate and
  all ordered layer snapshots. Native ABI offsets are corroborated by the
  pinned executable disassembly and `ToolOrdering.hpp` field layout.
- `ares-config.gdb`, `ares-probe-config/`: observed Rust pre/post normalization,
  prime gate and front-layer fields without modifying code.
- `audit-verification.py`, `verification-audit.json`, audit exit0:
  all completed outputs, exact banner-only equality, tool-command accounting,
  source/binary/input/raw-file SHA bindings. The first offline audit's erroneous
  assumption that the banner occupied line1 is retained as an infra exit1;
  correcting its location did not relax the permitted banner-only comparator.

Ares executable SHA256:
`065dee0f9586a9ec0ca49eeb7bd4b041e11c546b0dfe45cb2743081220c743ab`,
from tested tree `ebcfe03816609406beaf7106da433bb1cfb31231`, production-identical
through main `76b23505c8a89074ef04de0374b59ceb0986d21c`. Subsequent changes here
are documentation only. Every directed launch checked load<60, used CPU13,
and installed an any-worker exclusive-full START cancellation guard.

No new tests, Rust changes, comparator changes, full sweep or workspace rerun
is claimed for this docs-only verification. Existing tower-emitting positive
controls and their assertions are unchanged; this witness does not certify
active tower geometry, Type1, K3M, general multi-transform preparation or
Windows/macOS runtime. Review approval remains separate from verification.
