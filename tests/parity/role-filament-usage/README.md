# Runtime Native role-filament oracle

Source boundary: GCodeProcessor.cpp:1621-1639 role cache, :1641-1649 finalization,
:5722 role switch and :6036 result-map copy (OrcaSlicer 8500fcdcc).

The original shared source is tests/parity/validation/kobra3-centered/0.8.3mf,
SHA256 0dc37a07daedf1aa01a48832a2ff3b4e70f6e367075274391e42884a98db1b0c.
Positive explicit-plate role tests use the membership-complete wasm-plate1.3mf,
SHA256 f502b7a1ba819a479b7cbae99144fa68b6c5f1b658dbb731b9929c3b7742873c.
CLI runner/processor_migration/roles.rs obtains Native results at runtime through
OrcaRunner, with ARES_ORCA_CACHE_DIR and explicit ARES_GDB_BIN. The reader keeps
its pinned ELF SHA/layout, complete map keys/original f64 bits and final-cache
checks. No Native G-code or map is committed; ROLE_LIVE opt-in is removed.

The slice-mode oracle reads all eight roles from the SAME Native process that
writes plate_1.gcode. Input/G-code SHA links and both two-decimal footer mm/g sums
are checked before the map can serve as a reference. Public processed slicing
retains its full G-code control (only one generated-by line normalized), and
both public slicing and process_gcode results retain exact complete role bits.

The supplemental boundary test constructs a handwritten program body over a
runtime Native producer/config block, explicitly setting diameters 1.75/2.85,
densities 1.24/1.5 and filament_map 1,1. Same-role ToolChange, repeated/revisited
roles, support/interface/transition/tower, ColorChange/pause, retract/travel/wipe,
physical/virtual flush and EOF remain exercised. Its process-mode oracle has
footer_check=null: unequal-material/flush role totals do not share footer scope.
The same-run slice proof independently establishes reader reliability.

The WASM role test remains an ordinary public-result JSON pass-through test;
it does not read Native files. Required/optional availability follows the shared
Native policy, including visible SKIPPED for an omitted optional comparison.
See docs/architecture/native-artifact-cleanup-processor-spec.md.

The WASM positive explicit-plate test uses `wasm-plate1.3mf`, reproduced by
`generate_wasm_plate1.py`. It adds only plate1 object2/instance0/identify14
membership to the original shared input. Under Native importer semantics
(bbs_3mf.cpp:2058-2082), the original membership-empty input rejects --slice1
with -6/empty model in both modes, while --slice0 succeeds. The dedicated
membership-complete input admits --slice1 in both modes. Geometry, transforms,
settings and the original shared fixture remain unchanged. This preserves
Some(1) and every WASM statistics assertion; no import fallback is added.

The runtime Native role migration tests use the same complete input: their
producer's --slice0 and the public Some(1) select the same sole assigned object.
Input/G-code SHA bindings, full-output equality, footer sums, all f64 bit checks
and cache-boundary assertions stay unchanged. The procgcode reader/launcher and
original shared input are not modified.
