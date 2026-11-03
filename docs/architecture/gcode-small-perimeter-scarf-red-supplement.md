# Scarf caller-speed independent RED supplement

Before applying GREEN, production is restored to main1cf47e9a7 plus rebased
approved test/spec commits (HEAD5ebd8c60f). Pending GREEN diff is outside repo
under `${TMP_ROOT}/ares-tiq2-loop-red/scarf-independent-red/pending-green.patch`.

Source: `GCode.cpp:5808-5813,5896-5900,5919-5963,6515-6526,6564-6588`.
The loop caller selects positive small speed; sloped `_extrude` must not lose
that argument to a local slope-speed early return or skip the first-layer
ramp. Rust destination remains loop_paths/materialized/path/features only.

Frozen independent public byte-input case: original square STL, classic,
threshold5, small30%, outer120; scarf external/conditional0/min_length10/steps4,
scarf_joint_speed100%, slow_down_layers5, initial_layer_speed20. No geometry
changes; existing eight controls keep all original active settings. The seventh
key set is verified on the actual exported input before Native Slice.
Native non-first Outer witness is the source ramp:
`20+(36-20)*1/5=23.2` and `20+(36-20)*2/5=26.4`, F1392/F1584.
The complete compare_case PASS assertion remains authoritative, M73/footer
retained, independently of the feedrate witness.

Only the minimal features change is included: no requested caller speed keeps
existing slope default; positive caller speed bypasses the early return and
continues through the normal layer/ramp/cap chain. The local mm threshold
shortcut in `scarf.rs::scarf_speed` is explicitly deferred, not ported or cleaned
up here. Existing features tests only replace the former length parameter
10.0 by None; every assertion and expected value is unchanged.

First-attempt execution: Native and Ares exit0; nextest exits101 at the
complete-byte assertion. Native witness F1392/F1584 holds; Ares additionally
emits F2160 on sloped paths. Full-byte first difference remains the earlier
M73 insertion at byte988/line37; it is neither hidden nor normalized.
Input SHA1fe3f45bd2eecd49587fb5945544a8b33369065e4d33761badd51da5d02fd0b1,
Native SHA65e2d9dd9e968560f113cf560b799e8ff840ccc9a1149e21f508ff9488b1b10d;
receipt/request/output binding is checked. CPU0/seed0/runs1/fresh case cache,
16-thread ceiling, one shared300s deadline, no TIMEOUT or retry.
`gcode-small-perimeter-scarf-red-record.json` binds exact producer/resources
and external raw evidence paths. No private emitter/test-only production API.
