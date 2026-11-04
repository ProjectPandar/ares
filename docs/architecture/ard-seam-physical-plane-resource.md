# ARD: source-owned physical plane handoff, resource-control witness

Status: proposed, docs only; the
[SPEC](seam-physical-plane-resource-spec.md) precedes RED.
Base main 41f2; no production change. The old v2 NO-RED closure stands.

## Owner and decision

Orca8500fcdcc `PrintObjectSlice.cpp::compute_slice_z:24-47` and
`new_layers:49-75` own the physical plane; `Layer.hpp:256-260` stores it.
`GCode/SeamPlacer.cpp::gather_seam_candidates:1014-1040` consumes it,
`calculate_candidates_visibility:1042-1057` scores it, and
`place_seam:1500-1650` / `GCode.cpp::extrude_loop:5744-5775` consume the choice.

Use existing Rust `PlannedLayer.slice_z` through
`PostClassicTraversalPrintObject::physical_layers`. Replace only the height
scan compatibility shell in `seam_placement/runtime.rs::layer_slice_zs` and
`seam_placement.rs::apply_objects`; remove the scan, not the physical planner.
This is a libslic3r rewrite handoff, not an Ares-owned seam/pipeline design.

## Non-negotiable constraints

- Physical layer index, region-major record slot and raft-shifted layer id are
  different identities. Optional/duplicate region records do not define planes.
- Borrow canonical slice_z in physical order, preserving its existing f32
  candidate conversion. No reconstructed midpoint, region0/first-occupied,
  modulo/dedup/sort, fallback or admission guard.
- The d951 witness has identical oriented pre-seam loop geometry and candidate
  XY/angles; the wrong z changes visibility ranking at third-layer region1.
  No perimeter, loop-order, scoring or travel correction belongs in this slice.
- M73 postprocessing is CLOSED as unqualified for this witness. Complete strict
  comparison retains every M73/motion/arc/statistics/footer byte; no filtering
  or tolerance. Restored choices are an offline prediction, not whole GREEN.
- Public RED requires complete Native→API+real CLI success and raw
  API==CLI before target-equality assertions. The50→20 density-only control
  must whole-PASS. SPEC→RED→separate GREEN.
- Existing v2 rejected ancestry stays excluded. Residual owners are reported,
  not silently repaired. No committed probes, new API or core file I/O; preserve
  browser WASM/Windows/macOS/Linux portability.
- Rebase onto the advanced main before RED.
