# G4 first-layer sequence: bounded source-slice SPEC

Scope: `first_layer_print_sequence`. Existing other-layer count work is already merged and is not a new RED. SAFC Native nondeterminism is a separate slice and is excluded.

## Source and Rust boundary

`OrcaSlicer/src/libslic3r/PrintConfig.cpp:1109-1113` defines coInts `[0]`, range0..16. IDs in the configured first-layer owner are **one-based** at the point this consumer sorts region tools; only emitted tool identities become zero-based after `handle_dontcare_extruder` (ToolOrdering.cpp:248+). The original SPEC confused configured and emitted identities. `GCode/ToolOrdering.cpp:483-491` generates the first order only for the unspecified first-extruder sentinel, then handles dontcare tools. `apply_first_layer_order:498-514` applies the vector only when its length covers the collected tool list, ordering by first occurrence; absent IDs make the comparator return false. `generate_first_layer_tool_order:517-581` derives participating wall tools from supported first-layer slices and minimal area, then applies the vector. `Format/bbs_3mf.cpp:4429` imports plate metadata;7931-7934 exports it. Plate/project precedence must be proved from actual admission, not guessed from settings alone.

Rust destination: existing `ares-core/src/project_slice/gcode_emit/tool_ordering/first_layer.rs::apply_configured_order` and `handle_dontcare`, with existing project/plate settings resolution. They are the Rust rewrite shell for the named ToolOrdering methods, not a new sequencing pipeline. A failure may require a separate source-cited repair plan; no permission here to change shared ToolOrdering, master preferences, config fallback or sorting strategy. E2E lives in new key-local `ares-cli/tests/orca_parity/option_coverage/first_layer_sequence{,_context,_observation}.rs`, each under400LOC. Reuse existing runner/application/ZIP binding patterns, without mutating cached exports.

## Included fixture and lawful cases

Two separated cube10 instances on Ender3 .4 SEMM, two distinguishable PLA filaments at admitted211/223, relative E, by-layer, Manual same-hotend map[1,1], no support/raft/brim/skirt/tower. Each object has its own outer/inner wall filament (1 and2), so BOTH tools enter upstream minimal-area collection. A single cube with wall1/infill2 alone is not proof that both appear in this first-order collector. Authored object assignment and transforms must survive input/export verification; plate and project both carry the tested sequence. Use fixed Customize choice only; Auto/Customize enum classification is not promoted.

Finite cases (revised): empty vector; singleton[1]; complete[1,2]; complete[2,1]; covering duplicate[2,1,2]. Empty and short exercise the size gate; duplicate includes every used ID and tests first occurrence without missing-ID comparator ambiguity. No absent IDs, out-of-range tools, partial covering vectors of sufficient length or invalid IDs. Other-layer count0 and empty vector throughout isolate this owner.

Required observers: persistent actual tool state established by selected-tool/start/change templates AND corroborating positive-XYE object/wall extrusion roles; record initial tool, first-layer ordered wall tool visits, temperatures and later-layer visits on both sides. Complete reverse pair must change actual initial/first-layer tool consumption, not only a template comment/footer. Duplicate must match the reverse complete case's commanded order. Empty/singleton results come from Native, not a hardcoded default ordering. First-layer changes may lawfully propagate to later scheduling, so do not assert all later layers unchanged; compare each complete stream strictly and preserve actual schedules.

## Failure inventory before code

- Export may shrink filament palette, reconstruct identities, overwrite sequence or auto-arrange objects; keep pre/post input values and member hashes, assert both object tool assignments and separation.
- Plate metadata may override project sequence. Bind only cited settings/plate owners in a new ZIP, use identical bytes for Native and Ares, verify actual CLI loaded input and input hashes.
- Explicit first_extruder could bypass generation; prove sentinel/default entry path. If fixture start/template forces a different selected tool, report admission failure rather than weakening the observer.
- Wall-tool collection may see only one tool, unsupported tiny slices or equal-area ties. Two actual wall tools and reversed active consumption are mandatory; Native nondeterministic ties require raw SHA audit/adjudication, not tolerance or retries until PASS.
- Configured1/2 must not be confused with emitted0/1; the default[0] is an inactive/sentinel input, not a complete lawful permutation of two real region tools. Do not resurrect the already-fixed numeric footer issue or add a key exception.
- Sequence choices/templates can serialize without affecting physical order. Require positive extrusion corroboration and whole-file parity; no golden output or generated banner policy change.
- Native admission/compiler/harness errors are not production RED. No assumed regression: current code may already match these finite cases, in which case report test-only GREEN honestly.

## Admission investigation / fixture revision

The first test-first run: both modes admitted empty/short/forward/reverse with whole PASS, both actual outer-wall tools present, but reverse Native[1,0] request still emits[0,1]. Duplicate not reached; no production RED. Input project and plate both retain1,0 and Customize; Native footer also1,0, so no plate overwrite was observed. Print.cpp:2429 constructs ToolOrdering(*this,-1,false); by-layer GCode.cpp:2766 reuses this order, not sequential explicit initial_extruder bypass.

Decisive source trace: ToolOrdering.cpp:543 reads region outer_wall_filament_id, which is one-based;577 inserts ape.first without decrement;578 apply_first_layer_order compares against those IDs;188-247 handle_dontcare reorders one-based layer tools, then248+ reindexes to zero-based. `[1,0]` misses real tool2, so comparator499-514 returnsfalse for missing IDs and the requested reverse is not active. Rust first_layer.rs mirrors this ordering-before-reindex boundary. This corrects the original SPEC's index assumption; it is not an Ares fix.

The fixture revision changes only configured vectors to1,2 /2,1 /2,1,2 and short1, preserving emitted expected0,1 /1,0 and both actual wall/tool assertions. Run fresh five-case seeded/official afterwards. A successful reversal is mandatory before any production RED declaration; full option_coverage both modes remains required by registration rule.

## Deferred and exits

Defer Auto choice semantics, by-object printing, three-plus tools, unused tools/missing IDs, arbitrary vectors, malformed shapes, first-extruder override, automatic map grouping/master preference, support/raft/tower, other-layer ranges, multi-nozzle variants, and unbounded completeness. No inventory/plan promotion until an actual finite witness exists; unrelated rows unchanged.

Commit the tracer before Native directed seeded/official runs with fresh caches. Record actual role/tool observations and member hashes for all five inputs. Stop on any admitted production mismatch and write an owning-source repair plan before code. Run the full option_coverage group in both modes including tests::* whenever inventory/plan/sweep changes. Strict/fmt/WASM must pass; a production repair also requires workspace and the standard four-leg full run with named FAIL subset and input/channel audit. RED and one GREEN land together; no Native bypass or ignore.
