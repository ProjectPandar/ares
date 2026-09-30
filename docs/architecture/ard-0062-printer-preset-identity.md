# ARD-0062: Printer preset identity through original export

Upstream owner: `libslic3r/PrintConfig.cpp:5116-5120` declares the free-form
`printer_settings_id` string. `PresetBundle.cpp:219-223`,
`PresetBundle::construct_full_config`, assigns it from the printer preset name.
The native CLI's load-settings path, `OrcaSlicer.cpp:2034-2048`, likewise sets
it from the machine JSON `name`. Direct configuration-key injection alone is
therefore not evidence that the requested identity reaches a sliced project.

Rust destination is the existing native `option_coverage/preset_identity.rs`
fixture activation and authored free-form domain, with the existing project
wire/config-header output as the production seam. This extends ARD-0057's
source-owned identity adapter; it does not add a new Ares pipeline or fallback.

Include two distinct requested printer identities, exact export
requested/effective proof, final reference config literal, and complete
ordered original/Ares G-code comparisons. Retain the existing filament/print
identity regressions. Do not change geometry, printer model, nozzle options,
or template inputs to hide an identity mismatch. Adjacent preset-compatibility
or update behavior must be driven by an observed original error before any
fixture change; unrelated metadata remains unchanged.

Observed CLI rejection establishes one required fixture dependency:
`OrcaSlicer.cpp:2575-2581` matches process `compatible_printers` against the
selected machine's system name; `Utils.hpp:42` names rejection -17 as
`CLI_PROCESS_NOT_COMPATIBLE`. Renaming this fixture machine must rename only
matching old-name entries in that list. Preserve other entries and the actual
compatibility check; do not clear the list or force a compatible result.

Defer cloud/system preset lookup, vendor compatibility policy, all-printer
cross-products and material mapping. Existing profile selection remains a
temporary fixture adapter around the named upstream identity source.

The d2949771 complete scan reports this key INCOMPLETE with only a baseline
comparison. The newer 26dbef57 scan is still immutable and running separately;
its counts must not be rewritten using this slice's future targeted evidence.
