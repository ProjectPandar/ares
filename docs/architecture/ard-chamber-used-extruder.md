# ARD: chamber reduction owns no new membership registry

Status: accepted discovery; SPEC and test-only RED in scope; GREEN is a separate slice.

Owner: OrcaSlicer8500fcdcc `libslic3r/GCode.cpp:3008–3013,3037,3130–3133,3498–3499` and `GCodeWriter.cpp:192–212`. Destination: existing ares-core `project_slice/gcode_emit/machine/temperature.rs` chamber reduction and its three consumers, with private orchestration/machine context handoff only in the GREEN slice.

Non-negotiable decisions:

1. Activation OR and independent temperature max consume exactly the established writer IDs. Reuse `ExtruderRegistry::ids()` already handed through by frozen exhaust53890b780. No parallel registry, object/model-derived substitute, fallback scan or public API shim.
2. The current object-based chamber reducer is a temporary incomplete shell around upstream behavior, not an Ares-owned pipeline. Do not change unrelated nozzle/bed uses of the shared object collector as incidental cleanup.
3. Preserve upstream indexing, gates, auxiliary fan commands, overall placeholder, ordering and complete config serialization. Keep all output bytes in the existing strict comparator, including estimator consequences.
4. Four accepted input archives are immutable. Native is the runtime oracle; no Native command/count constants. Test-only RED must produce two real comparator failures and two control passes in each existing Native mode, plus raw API==CLI evidence.
5. get_at is deferred, not closed or fixed. No production work until the separate GREEN slice; integration follows the combined exhaust/chamber rebase.

Included and deferred behavior, exact GUI entry citations, failure inventory and exit criteria are in [SPEC](chamber-used-extruder-spec.md). Browser WASM core remains platform-neutral; no new filesystem/UI/OpenGL/terminal behavior enters core. This stage adds no production code.
