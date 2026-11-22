# Sequential non-BBL automatic filament map publication

## Upstream boundary

This slice rewrites the static sequential automatic map publication in
OrcaSlicer v2.5.0, revision `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.

| Upstream owner | Included behavior | Rust destination |
|---|---|---|
| `Format/bbs_3mf.cpp:4674–4681`, `OrcaSlicer.cpp:6924–6926` | Deserialize the concrete plate filament map mode and apply the plate config over the project, before CLI extras. | Existing `project/plate_config.rs::apply_plate_config_overrides`; no change. |
| `Print.cpp:3695–3739` | After collecting actual sequential print usage, compute the static group result and write its maps back in automatic modes, including on multi-head printers. The sequential Manual paths do not write back here. | `project_slice/gcode_emit/tool_ordering/recommend.rs::build_recommended`: extend entry to its existing publication block for static non-BBL, one-nozzle-per-head multi-head sequential automatic prints. |
| `GCode/ToolOrdering.cpp:1752–1767, 1796–1805` | The non-BBL automatic grouping result gives each configured filament a nozzle entry, using the identity prefix and master-head default. | Existing `options/project_variants/filament_maps.rs::update_static_filament_maps`, unchanged. |
| `Print.cpp:3725–3739, 4661–4680`; `FilamentGroupUtils.cpp:348–359` | Preserve the configured extruder and volume values for unused filaments; if their seed lengths differ from the derived count, first synthesize the original defaults. Publish the complete derived nozzle map. | Same existing static publisher and used-only merge; propagate its refreshed config views via the existing checkpoint. |
| `GCode.cpp:8272–8305` | Export the full print config, including the applied variant index map. | Existing config-block reconstruction, unchanged. |

`tests/parity/sequential-first-layer-nozzle-temperature/second-tool.3mf`
has project-level `filament_map_mode = Manual`, but plate 1 in
`Metadata/model_settings.config` supplies `Auto For Flush`. OrcaSlicer
reads the effective plate value at the import and overlay points above;
`Print.cpp:3706, 3718` tests that effective mode at publication. Ares also
exports `Auto For Flush`. The fixture configures two tools, emits only
the second tool, and expects these four footer values:

```text
filament_map = 1,2
filament_map_2 = 0,1
filament_nozzle_map = 0,1
filament_volume_map = 0,0
```

Ares currently publishes the complete values in the otherwise identical
by-layer case. The multi-object by-object case with multiple heads never
enters the existing publication block, leaving one-entry nozzle and
volume defaults. Public 3MF slicing does not use `FilamentSlotState` for
this path. The defect is missing publication, not a palette resize or a
formatter defect.

## Scope and relationship to existing publication

For multi-object by-object printing, extend the existing entry condition
only for non-BBL static automatic mapping on multi-head printers with one
nozzle per head. This covers `Auto For Flush` and `Auto For Match`, which
use the same static non-BBL branch. Keep the current single-head and
one-object admission conditions unchanged. No new assignment policy or
footer-specific logic is needed.

The change touches the same file and function (`recommend.rs::build_recommended`)
as the non-BBL used-only map rewrite `971b136e4`, but not the line it added
(the `is_bbl` argument to `update_static_filament_maps`). It changes only
eligibility for configurations previously excluded by `ByObject` with
multiple objects. The existing by-layer call, the recommendation and its
seed/used-only merge, and the BBL and multi-nozzle grouping dispatch are
unchanged. Mixed-slot handling, dynamic per-layer reorder, sequential
Manual and Nozzle Manual, and BBL multi-head sequential publication remain
with their current source boundaries. No public API change is included.

## Verification seam

Name the two-tool test for its effective `Auto For Flush` mode. Check all
four exact footer fields through the pinned Native CLI, the public Rust
slicing API, and the Ares CLI on the same plate. Keep the original restart
span assertion in the sequential case. Compare the same four fields in a
non-sequential two-tool control built at runtime from the same project.

Strict whole-file comparison is deferred because the first-to-second-layer
transition already differs before the footer: Ares emits `G10 S207 P0`
in addition to `G10 S207 P1`, while Native emits only the latter. Do not
rewrite or normalize the extra command. Do not commit Native outputs.

Pair the test-only and implementation revisions with the official and
seeded Native runners across filament-map, flush, tool-ordering,
multi-tool and multi-nozzle families. Include both landed
`non_bbl_auto_map` cases explicitly: `non_bbl_auto_map_keeps_three_unused_filament_assignments`
and `non_bbl_auto_map_changes_only_the_used_second_filament`. Previously
passing cases must remain passing. Run formatting, strict workspace
all-target Clippy and core/WASM checks.
