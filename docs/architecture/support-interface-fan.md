# ARD: Support-interface fan requests at the project G-code seam

Status: Implemented; both reference suites and quality gates pass.

## Source owner and Rust destination

OrcaSlicer v2.4.2 `GCode.cpp::_extrude`, its
`append_role_based_fan_marker` / `apply_role_based_fan_speed` lambdas
(around6889–6921), and the constant/variable extrusion calls own support
interface START/END markers independently of `enable_overhang_bridge_fan`.
`GCode/CoolingBuffer.cpp::parse_layer_gcode` and
`CoolingBuffer::apply_layer_cooldown` (support control around747,777–778,
802–803,870–879,992–994) own layer-gated fan requests, fixed speed (not the
part/overhang ramp), priority and restoration. `PrintConfig.cpp` defines
`support_material_interface_fan_speed` as a filament integer list in -1..100.

Rust destinations are the existing `project_slice/gcode_emit/motion/fan.rs`,
its state/options assembly, and `cooling.rs` / `cooling/role_fans.rs`.
The existing deferred-role marker format remains only a temporary internal
transport shell for these source-owned requests, not an Ares fan policy.
No slicing geometry, support fill connection, AdaptivePA, timelapse, tower or
bridge-direction behavior is included.

## Decisions

- -1 disables the interface override; 0 is an explicit fan-off request.
- Interface control is independent of overhang-bridge cooling enablement.
- Emit START on entry and on continuation into a new layer chunk; END on exit.
- Cooling request state is scoped to the flushing LayerResult. Closed first
  layers do not activate interface requests; END only emits for an active
  interface request. An inactive END falls through `apply_layer_cooldown`'s
  ordinary-line branch and retains the literal `;_SUPP_INTERFACE_FAN_END`
  comment in native output; preserve it rather than stripping it. Preserve
  ordinary role-END semantics unchanged.
- Interface speed is fixed, including values below the layer baseline; do not
  ramp it or take its maximum with the baseline. Overhang/internal-bridge
  requests retain upstream priority during sequential marker transitions.
- Restore the active higher-priority request or layer baseline on interface
  END. Existing force-resume behavior must observe any active role request.
- Use the active filament's typed configuration. No fallback global filament
  index, new user-facing option, runtime filesystem or platform-specific core
  code is introduced.

## Included / deferred and acceptance

Included: the missing support-interface producer/consumer path and strict
public 3MF-to-complete-G-code regression using the existing overhang fixture.
The min/max/seeded Option domain stays unchanged. Add focused public controls
for zero, disabled, below-baseline, overhang-control-disabled and closed-layer
behavior, with native interface extrusion and fan-command witnesses.

Deferred: ironing fan activation (adjacent but distinct source role), full
multifilament fan lifecycle coverage and other Option-range completion.
Do not claim existing inert Option PASS rows establish these deferred behaviors.

No comparator edits, unsupported guards, silent skip or private implementation
pinning. Both seeded and official modes must compare complete output.
Workspace Nextest, Clippy, rustfmt and core/adapter WASM compile must pass.
