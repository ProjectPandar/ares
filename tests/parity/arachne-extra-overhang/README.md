# Arachne extra-overhang application parity

SPEC5530ca55b. Owning upstream:
PerimeterGenerator.cpp1087–1113 apply_extra_perimeters;
process_arachne2521 missing call. Existing Classic call1672, generator,
numerics and ordering are unchanged. Rust destination is Arachne
materialize_record after ordinary fill append, before no-overlap.

Actual official CLI-exported rotated stepped model. All five archives
copied byte-for-byte from the original discovery; manifest.json pins
whole-file/per-ZIP SHA256 and retained ordinary GUI settings. Primary
arachne-control→arachne-positive differs in exactly one ZIP payload,
project_settings.config, and one key extra_perimeters_on_overhangs0→1.

Four controls: extraOFF, Arachne detectOFF/extraOFF, Arachne
detectOFF/extraON, Classic detectON/extraON. Classic detectOFF is NOT a
case here: it belongs to an independent SeamPlacer candidate.
Common aligned/staggerOFF/scarfNONE/wipeOFF/density0%. No grouping,
seam/travel/wipe or geometry-generator fix belongs to this chain.

Tests use main runner/native_api_cli.rs::NativeCase: fresh Native complete
stream, validated public API, bare CLI, full-stream comparator and raw
API=CLI. Only Native availability condition; no borrowed gate, copied
helper, handwritten runner, ignored test or private slicing boundary.
Native configuration and target actual motion G1 X117.223 Y117.223 Z1
are active premises, never movement-only acceptance filtering.
Targeted filter: `test(arachne_extra_overhang_call_)`.

RED must produce4controlsPASS/1positiveFAIL in each mode, through both
public boundaries. GREEN all5 complete cases must pass in official and
seeded. NativeCase preserves inputs, all outputs, launches and
comparison.json under ARES_PARITY_ARTIFACT_ROOT.

A local probe GDB run matched12 PG records/24 phases,
including layer2 extra3→4 and cut fill contour7053115/-7053111 with
identical complete raw inputs. This is local-probe evidence, not a
current-main integration claim. Gates: targeted dual, fmt,
workspace/all-target clippy-Dwarnings and browserWASM.

Reachability, with `OUT` an external directory:
`$ORCA --slice 0 --outputdir "$OUT/arachne-positive" arachne-positive.3mf`.
