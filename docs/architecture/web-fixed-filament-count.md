# Fixed filament slots: PresetBundle rewrite slice

## Authority and scope

User request: Snapmaker U1 and other non-SEMM multi-extruder printers keep their hardware filament slots; users cannot add or delete slots. Source authority is OrcaSlicer v2.5.0, `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`, checked out at `/home/indexyz/ares-coord/orca-source-250`.

This is a rewrite of the following upstream boundaries, not a new Ares filament-count policy:

- `OrcaSlicer/src/libslic3r/PresetBundle.cpp:5400-5424`: `get_printer_extruder_count` and `min_filament_count`. The nozzle vector supplies the hardware count; SEMM has minimum one, otherwise the minimum is the hardware count.
- `PresetBundle.cpp:5426-5436`: `update_filament_count` grows an FFF selection to its minimum, copying the last preset or the first visible preset when empty. It does not truncate longer selections.
- `PresetBundle.cpp:8598-8671`: `update_multi_material_filament_presets` pads presets and resizes `flush_multiplier`, `flush_volumes_vector`, and the per-nozzle square `flush_volumes_matrix`, preserving old entries and initializing new off-diagonal entries from the per-filament load/unload volumes.
- `OrcaSlicer/src/slic3r/GUI/Plater.cpp:6574-6610`: `should_show_SEMM_buttons` / `show_SEMM_buttons`. SEMM or BBL permits add; delete and flush controls also require more than one slot. Other printers have fixed slots.
- `Plater.cpp:3740-3760`: multi-material SEMM/BBL edit opens the filament action menu; fixed and single-slot printers open the filament settings tab directly.
- `Plater.cpp:5826-5841`: adding is bounded by `MAXIMUM_EXTRUDER_NUMBER`; deletion stops at one slot.

## Deduplication

Base is `origin/main` at `8e42def8eaef06bb567e5d5c505a2daadb9fe4fb`. Grepping all origin branches' core/WASM/sidebar code found no `min_filament_count`, `update_filament_count`, or `should_show_SEMM_buttons` port. Search evidence is `/home/indexyz/ares-evidence/web-fixed-filament-dedupe.txt`. Candidate registration contains this W1 assignment only for this boundary.

W2's accepted `5e49bd563` uses the loaded 3MF filament count for an empty profile selection. It is adjacent, not duplicate: it does not enforce the printer minimum or implement sidebar control policy. Preserve that project-derived selection behavior. Existing web H2D/nozzle mapping remains owned by `state/extruders.ts`; no second extruder-map or nozzle-volume rule is introduced.

## Rust destination and architecture decisions

The pure preset-layer API owns the count rule, padding, flush layout, and SEMM/BBL control policy. It accepts resolved printer configuration and current selection/flush data, and returns normalized selection/layout plus the control policy. Catalog callers resolve printer and first-visible preset information before invoking it. Browser code does not infer the hardware slot count from printer names or duplicate the count rule.

Expose the API through `ares-wasm` and the existing `ares-wasm-js` catalog/worker transport. Keep it filesystem-, terminal-, and UI-independent on WASM, Windows, macOS, and Linux.

The web calls it when selecting/restoring a printer and when loading a project with an active preset selection. These map to `PresetBundle.cpp:3939,4101` selection restoration, project-config normalization (`update_multi_material_filament_presets` callers), and printer changes through `Tab.cpp:5867-5880,6536-6551`. `state/actions.ts` already invokes `updateCompatible` after project load; reuse this entry rather than creating a parallel lifecycle. A selection update must retain the existing stale-request identity check.

The sidebar renders all returned slots, even when a loaded project has fewer filament names. The selection, settings/slicing request, colors and flush-layout overrides must agree on the normalized count. Fixed printers do not show add/delete/flush actions; edit goes directly to the tab. The current `Add/Remove filaments` combo entry manages the *installed preset set*, not slot count, and must remain available. Per COORD's scope choice 2, this slice does not create the currently absent SEMM/BBL add/delete/flush controls.

Existing scaffolding (`presetFilamentSlots` and the web's `Math.max` selection sizing) is only a compatibility shell for project slot discovery. Hardware-minimum logic moves into the upstream-owned Rust preset API rather than extending that shell with JS printer rules.

Do not truncate a pre-existing selection longer than the hardware count: upstream enforces a minimum, not an exact-count truncation. Fresh U1 selections still become exactly four slots; the UI prevents further manual growth or deletion.

## Included and deferred

Included: FFF minimum count, last/first-visible padding, flush data resize and preservation, printer switch/session restoration/project-load wiring, fixed-slot rendering and direct settings edit, and U1 slicing through the normalized selection.

Deferred: adding the currently absent SEMM/BBL add/delete/flush controls and multi-material action menu (their future policy must follow `Plater.cpp:3740-3760,6574-6610` `show_SEMM_buttons`); SLA material selection, physical AMS synchronization, virtual/mixed-filament creation, hardware discovery, and unrelated H2C/filament-group logic. Do not change wipe/prime-tower generation. Flush layout is the cited PresetBundle state update only, not a tower feature.

## Failure modes and agreed test seams

Use public core/catalog APIs and Playwright; do not test private helpers. Runtime Native evidence comes from the CLI only where it reaches the same boundary, otherwise a chain-probe observer of `PresetBundle::update_multi_material_filament_presets` at the pinned revision. No frozen Native result goldens.

Failure modes to cover before implementation:

1. One U1 filament remains one slot, or pads with the wrong preset: Native versus public API must yield four last-preset copies.
2. An empty selection has no padding source: use the catalog's first visible filament.
3. SEMM/BBL incorrectly inherits a four-slot hardware minimum: SEMM remains minimum one; BBL controls remain mutable.
4. An existing longer selection is truncated: retain it, matching Native.
5. Flush arrays retain a one-slot layout or overwrite preserved cells: U1 has four multipliers, eight per-filament entries and four 4x4 matrices; verify values against Native.
6. Browser stores four slots but renders only the project's one slot, or slices the old one-slot selection.
7. Fixed printers expose add/delete/flush, or edit opens a mutable action menu rather than settings.
8. Switching to SEMM/BBL incorrectly applies U1's hardware minimum; switching back or restoring the session loses slot state.

Commit order: this source-cited spec, then an observable failing test commit, then minimal implementation. Add tests to their family modules; `orca_parity/runner.rs` must have net growth zero.

## Exit evidence

- Runtime Native/core parity with a repeatable input/observer manifest and durable artifacts.
- Playwright: select U1, four slots and no add/delete/flush controls, edit opens tab; switch to SEMM/BBL and verify no U1 hardware minimum leaks; U1 slice completes. Screenshots and result JSON under `/home/indexyz/ares-evidence/web/fixed-filament-count`.
- CPU affinity `36-39`; browser port `4194`; no use of CPUs `0-11`.
- `cargo fmt --all -- --check`, strict workspace/all-target Clippy, wasm32 check, TypeScript and oxlint.
