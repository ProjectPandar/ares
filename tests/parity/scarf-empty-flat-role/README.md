# Remaining-flat scarf role parity

Approved boundary: ExtrusionEntity.hpp485 ExtrusionLoop::role;
ExtrusionEntity.cpp471–505,550–554 ExtrusionLoopSloped construction and
clip_front; GCode.cpp5940–5949 whole-loop/oversized minimum-length cap.
SPEC55d40a766 re-establishes approvedfafec2987. Rust destination is only
scarf.rs::build's role predicate; geometry and clipping arithmetic stay.

Five actual official CLI exports copied byte-for-byte from the original
Native discovery. manifest.json pins whole-file/per-entry SHA256,
retained settings and original export paths. Four one-payload/one-key
pairs were verified while freezing:

- control→positive: seam_slope_entire_loop0→1;
- positive→no-inner-control: seam_slope_inner_walls1→0;
- positive→zero-gap-control: seam_gap10%→0%;
- control→length100: seam_slope_min_length20→100.

All use aligned seam, stagger OFF, wipe OFF. No malformed scalar arrays
or post-export edits. `wipe=["0"]` is the legitimate per-extruder vector.

Formal tests use main's runner/native_api_cli.rs::NativeCase, public
validated API and bare CLI, complete streams with only the canonical
single generator identity/date exception. No copied helper, manual flow,
private seam, ignored test or borrowed PROFILE_ENTRY gate. Native
availability is the only environment condition. Targeted filter:
`test(scarf_remaining_flat_role_)`.

RED: both positive and length100 fail, three controls pass in each fresh
Native mode. Native positive actual XY witness G1 X119.325 Y100.755 Z.8
is an active assertion, not movement-only acceptance. GREEN: all5 cases
must completely pass in official and seeded, raw API=CLI. NativeCase
keeps every stream, CLI logs and comparison.json under artifact root.

Prior authorized local-probe GDB proves positive crop80000 scaled units
versus old29840001.6, control remains8000000. Its evidence is not a new
current-main integration claim. Authorized pre-window gates are targeted
dual mode, fmt, workspace/all-target clippy -Dwarnings and browser WASM.
No FULL/integration RR; window rebase/FULL remains separate. Temporary
lintfaf4ff649 must be dropped because its original231bd11cc is on main.

Reachability can be repeated with the official CLI:
`$ORCA --slice 0 --outputdir /tmp/scarf-role-positive positive.3mf`.
Original export/slice/repeat receipts: /tmp/w5-next-entire-min1/verified.json.
