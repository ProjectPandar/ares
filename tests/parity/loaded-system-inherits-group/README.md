# Loaded-system inherits_group public parity fixture

SPEC: `6c38e182fd8f22388833cc55b225c26dbb805dab`.
Upstream pin: `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
Owners: PresetBundle.cpp::construct_full_config69-236, CLI::run system source
metadata2038-2047/2079-2086, group2838-2842/2981-2985/2189-2193/3161-3163,
GCode.cpp::append_full_config5591 onward. See architecture SPEC/ARD.

`single.3mf` and `positive/*.json` are exact retained rKLPnw single-control
inputs, previously archived in identical-loaded-single-F2JO. The 3MF selects
Arachne/sandwich/auto_brim/reversal0; both slicers receive the same loaded files,
explicitly Classic/inner-outer/no_brim/reversal1/internal1. No settings regenerated
from the current vendor catalog; Native does not resolve these flattened parent
files. Every loaded file has from=system and a nonempty raw inherits name.

`generate_control.py` reproducibly derives `control/*.json`: only the three
raw inherits strings become empty, all other bytes unchanged. Same3MF/settings
IDs/from/system/geometry. `manifest.json` pins all seven input digests.
No Native G-code output is committed.

Failure modes at the agreed public seam: admission/source drift, incomplete or
unstable Native output, changed Native control behavior, API/CLI raw mismatch,
and actual effective-config/full-output mismatch. Neither matching effective
config nor a particular first-difference position is a precondition to RED.

One test, `loaded_system_inherits_group_preserves_cli_source_status`, runs exact
positive +fresh positive repeat +control with private Native caches. Existing
`OrcaRunner::from_env`/stage capture and Native direct loaded-settings route are
used; API is slice_with_load_settings and CLI --load-settings/--load-filaments.
All three receive identical source bytes, no extra overrides, placementNone,
plateNone/Native slice0, deterministic metadata/contextseed0.

Native positive/repeat normalized complete streams and API raw streams must be
stable. Native positive/control must be generator-only byte-equal; control must
pass full comparison. Actual effective configs are recorded without filtering.
Public API/CLI must be raw-equal. Positive full generator-only equality is the
RED assertion; no footer/metadata/motion/arcs/M73/statistics may be removed.

Artifacts: fresh per-leg source copies, full native/ares/cli.gcode, native
cache producer/request receipts and command status/stdout/stderr, complete config
maps/digests/comparator results in loaded-system-result.json, plus three-leg
loaded-system-summary.json. Run via canonical official/seeded environment with
ARES_PARITY_PROFILE_ENTRY_REFERENCE=1, ARES_REQUIRE_ORCA=1 and fresh
artifact/cache roots. This is a test-only RED; no production correction.
