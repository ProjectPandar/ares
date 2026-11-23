# Keep loaded 3MF instances in place on a plater printer switch

## Upstream boundary

OrcaSlicer v2.5.0, `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`:

- `src/libslic3r/PresetBundle.cpp:5463–5478,5534` — `full_config` and
  `full_config_secure` use `PresetBundle::full_fff_config` for FFF. The new
  printer config is applied to the loaded model rather than to a new import.
- `src/slic3r/GUI/Plater.cpp:13025–13032,14533–14546,21665–21708,14995–15045`
  — the sidebar selects the printer, applies the composed config and changes the
  bed geometry. The switch updates the object list but does not arrange it.
- `src/slic3r/GUI/PartPlate.cpp:6423–6440,7083–7102,4180–4235` —
  `PartPlateList::reset_size` calls `update_all_plates_pos_and_size` with
  `adjust_position=false`. `PartPlate::set_pos_and_size` receives this as
  `with_instance_move=false` and does not change the loaded instance offsets.
- `src/OrcaSlicer.cpp:1899–1909,4350–4354,4371–4382,4414–4443,4602–4690`
  — the CLI instead reads old and new bed extents, classifies
  `shrink_to_new_bed`, then translates instances and adjusts wipe-tower
  coordinates. The larger-bed case uses the difference between the old and
  new bed centres.

## Rust destination and scope

Keep `Project::from_3mf_with_profiles` in
`crates/ares-core/src/project/domain/profile_input/three_mf.rs` as the shared
config and model assembly boundary. It already receives the composition
chosen in `crates/ares-core/src/profiles/entry.rs`: explicit GUI selection
supplies `full_fff_config` even if no groups are serialized, while the CLI
entry does not. Split placement at that entry distinction, not at preset
names, bed dimensions or the 3MF family. For explicit GUI composition, keep
loaded instance offsets after applying plater edits; do not run the CLI
translation block, including its wipe-tower coordinate shift. In CLI
composition, keep the existing BBL-family check, bed classification,
instance translation and wipe-tower offset unchanged. Keep import scope,
configuration composition, and STL placement unchanged. Update the two
adjacent CLI citations in `three_mf.rs` to the pinned bed reads and translation
listed above. The shared profile overlay remains an adapter for the upstream
composition paths; no new placement stage is needed.

Defer GUI per-plate wipe-tower clamping (`PartPlate.cpp:6331`,
`Plater.cpp:14539–14543`), per-printer stored/default `curr_bed_type`
(`Plater.cpp:3876–3905`), and how instances outside the changed bed are
classified or presented. The in-bounds case does not specify any of those
behaviors.

## Verification

1. Change the existing U1 slice case in `web/e2e/printerSwitchFilaments.spec.ts`
   to slice a saved-project reference with the original build-item transforms.
   Select Snapmaker U1 in the app after loading the A250 project. In the
   reference, pin the selected FFF config using the preset-select probe and
   keep the input model and its positions. Slice that already-composed 3MF
   with the pinned Native CLI without a machine overlay or arrange; this
   reproduces the GUI's geometry at the slicing boundary, not the live GUI.
   The objects fit both beds, and the saved project keeps the session's
   prime-tower setting and coordinates. Keep the existing normalized
   whole-file equality assertion; do not add another comparator.
   Before the change the browser applies the CLI half-delta of (+20, +10) mm
   to `hole.stl`, `wide.stl` and `neck.stl`; after the change the slice must
   equal Native with all three offsets preserved.
2. Run the unchanged `embedded_script_whole_file_matches_native` and
   `embedded_script_without_old_bed_height_matches_native` CLI-composition
   cases in both official and seeded reference modes on the starting and
   final revisions. In each mode also compare the deterministic Ares G-code
   bytes directly between revisions, without normalization, to establish
   that the CLI shift and emitted output did not change.
3. Check formatting, workspace Clippy across all targets, the WASM build and
   web static checks. Keep the existing browser test identity.
