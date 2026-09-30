# Overhang segmentation implementation plan

1. Preserve complete KSR strict RED and locate the first movement witness;
   verify: unchanged comparator, retained streams, repeated Ares output hash.
2. Capture the cited native/Rust producer boundaries read-only to guide
   public minimization; verify: normal inferior exits, complete G-code, valid
   capture values and actual binary identities. Exclude failed probes.
3. Shrink one input dimension at a time to a public closed-mesh RED; verify:
   complete ordered comparison and the same divergent producer boundary.
4. Add the public regression before implementation, repair only demonstrated
   upstream behavior, remove superseded private pins; verify: RED/GREEN and
   original KSR rerun without acceptance-comparator changes.
5. Run consumer/workspace/style/Tier-1 gates and independent six-axis review;
   fix and re-review findings, commit code/docs together, then push.

Current status: original replay17956 exits100/18.791s at
`/tmp/ares-ard79-ksr-ordered-current/case-YF3e5N`. Rust boundary captures34164
and5713 exit0 with normal inferior exits, capturing20 path input points,
19 previous-boundary segments and42 processed points on layer_index226.
Complete diagnostic streams are `/tmp/ares-ard79-rust-capture.gcode` and
`/tmp/ares-ard79-rust-boundary-capture.gcode`.

Native attempt36770 stops on an optimized lambda variable and is excluded.
Exact-signature replacement59827 also stops early: its C++ aggregate return
has no debugger value address. Exclude it despite debugger exit0. A third
capture uses the x86-64 aggregate-return ABI and completes0 with a normal
inferior exit (handle68259). Its20 path inputs match Rust exactly after
unscaling; both return42 points with three integer XY differences. Full
native output is `/tmp/ares-ard79-native-sret-capture/plate_1.gcode`; capture
is `/tmp/ares-ard79-native-overhang-sret.log`. Native previous-boundary
comparison remains pending. Do not treat a process start
or GDB exit status alone as proof of a completed native slice. Public
minimization, causal proof, production repair and submission gates remain
pending at that investigation checkpoint.

Further evidence: boundary81624 completes0 with a normal native exit; its
19 previous-boundary lines equal Rust exactly. Extended Rust capture56898
using the unoptimized build51115 completes normally and preserves complete
G-code equal to optimized output except the generated timestamp. Integer
intersection reconstruction loses sub-micron positions before segmentation;
native preserves floating intersections. Failed optimized captures16069 and
61443 are excluded. Public stepped-bar explorations are insensitive so far;
do not replace the KSR regression with their passing comparisons.

Next-boundary98098 completes0 with normal native exit. A two-layer11-vertex
closed mirrored loft at input XY(110,110.000105) is sensitive; a four-vertex
bar simplification is not. Strict replay completes100/0.311s, pair
`/tmp/ares-ard79-loft-public-red/case-n1g9K2`. The24 earlier insensitive
angle/length fixtures pass the unchanged strict comparator (0.283s).

Permanent live test18041 completes100/1.354s after20 canonical setting
checks and XY placement verification; complete pair
`/tmp/ares-ard79-permanent-public-red-output/case-9NZ2G7`. Test setup failures
for missing `arc_fitting` and auto-drop Z expectations are not output REDs.
Use `enable_arc_fitting`; XY placement is the intended input contract.

Production now uses floating AABB candidate intersection queries without
early integer truncation, preserving existing integer queries and tree
pruning. Public regression73698 completes0/1.425s with full artifacts at
`/tmp/ares-ard79-public-green`. Workspace72018, clippy12304 and original
KSR69601 were pending at that checkpoint.

Clippy12304 and Tier-142572 finish0 (all four core targets plus WASM adapter).
Original KSR69601 finishes100/42.184s; pair
`/tmp/ares-ard79-ksr-after-float-intersections/case-BV7aDL`. Strict first
difference remains M73 line9946 with6321717 normalized bytes each. Diagnostic
first non-progress mismatch advances to line221229/Z52 travel/helical-lift
path ordering. Do not treat that diagnostic as full ordered acceptance.
Workspace72018 finishes0: 7460 PASS /9 skipped in256.939s. Eight live
complete-output consumers6493 finish0/1.969s; artifacts remain at
`/tmp/ares-ard79-public-consumers`. The24 cached stepped fixtures finish0
in0.295s after repair. Fmt and diff checks finish0. Initial independent
six-axis review reports no required production changes. Final independent
review approves the immutable ten-file patch with no required fixes;
re-confirm this review-status documentation update before commit/push.
Native generic equal-key sorting and 3D/scarf behavior remain outside this
XY slice. Original strict KSR and global printer/option gates remain open.
