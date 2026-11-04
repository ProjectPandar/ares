# SPEC: printer_technology application-phase bounded domain (slice 1)

Docs-only SPEC; no SLA slicing implementation. Specified on
`96328f269d64165c02296ba9924f5c6ee972777c`.
Originating audit: `96328f269`,
`2026-10-14-option-full-current-audit.md` in
`docs/architecture/`.
The original execution was `f637b4122`.
No build, Native invocation or comparator change was made for this SPEC.

## Immutable observed failure

Both original machine JSON requests contain `printer_technology="SLA"`.
Native Export succeeded, but each exported 3MF's
`Metadata/project_settings.config` contains `"FFF"`. Generic application
verification failed with `Application/ApplicationMismatch` before Native
SLA slicing or Ares slicing. Original domain status stays **FAIL** in both
modes. This is not evidence of a missing Ares SLA algorithm.

Retained discovery evidence:

- `printer-technology-{official,seeded}-attribution.json` identifies exact
  requested JSON, exported 3MF, command and cache receipt paths.
- `audit-failures/printer-technology/{official,seeded}/` retains copied
  originals; never overwrite these with future corrected cases.
- Requested machine SHA256 (both modes):
  `ad9683bd4d482fb49cb5a318c35ed713aa43e47d12a91943d5f7d4cb71658ccf`.
- Official exported 3MF SHA256:
  `bda40e8f737eba48a78fb1d7c540a159dd285cc1221a68d22a5add78afb0ab3f`.
- Seeded exported 3MF SHA256:
  `eb4c4a2df0c235860206c6c826cb159be63ad27860d35c59d3b9dc0097c50c06`.

The retained official command loads machine/process JSON via
`--load-settings`, filament JSON via `--load-filaments`, then
`--arrange 1 --export-3mf <path> cube10.stl`. The receipt's Export success
and producer identity do not certify exact application of SLA.

## Source-owned parsing and resolution chain

Vendored `OrcaSlicer/src/`:

1. `libslic3r/PrintConfig.cpp::PrintConfigDef::init_common_params:674-684`
   registers FFF/SLA and defaults to FFF. Enum membership is schema legality,
   not CLI admission or supported slicing technology.
2. `OrcaSlicer.cpp::get_printer_technology:390-394` reads the **typed option**;
   absent option returns `ptUnknown`. It does not default absent input to FFF.
3. `OrcaSlicer.cpp:1326` starts local technology from CLI `m_config`.
   `:1854-1859` resolves imported-model config when local is unknown, and
   rejects a known conflicting imported technology (`CLI_INVALID_PRINTER_TECH`).
4. Settings-loading path `:2023-2106` parses each JSON into DynamicPrintConfig.
   Crucially machine branch `:2070` moves `config` to `load_machine_config`
   (process branch `:2096` likewise moves to `load_process_config`), then
   `:2100-2106` calls `get_printer_technology(config)` on that moved-from
   local, not on the stored machine/process configuration. Reading valid
   moved-from state is not equivalent to observing the requested SLA option.
   Do not infer its concrete retained map contents merely from C++ move syntax;
   the original SLA→FFF receipts establish this input's effective result.
5. Filament loading has another technology consistency site `:2180-2185`.
   Precedence and known/unknown states must be recorded per input channel,
   not conflated into a single enum-value motion sweep.
6. `:3511-3512` converts unresolved `ptUnknown` to FFF. `:3619` **overwrites**
   `m_print_config`'s technology with the local resolved technology, after
   applying command-line extra config and normalizing FDM (`:3614-3617`).
   Thus carrying SLA in an input JSON does not guarantee SLA survives export.
7. `:3627-3633` accepts the FFF configuration branch and otherwise records
   `CLI_INVALID_PRINTER_TECH` then exits. The SLAFullPrintConfig declaration
   and SLA branch implementation are commented (`:3623-3624,3634+`).
8. `libslic3r/PrintConfig.cpp::DynamicPrintConfig::validate:8866-8883` defaults
   an absent option to FFF, validates FullPrintConfig for FFF, but returns an
   empty error map for other technologies. That empty map is not SLA slicing
   support and does not override the CLI's explicit non-FFF exit.

Source conclusion: request deserialization, local CLI technology resolution,
exported effective technology, config validation and slicing admission are
five distinct facts. A schema-legal SLA request cannot earn SLA application
coverage from a successful FFF export.

## Does a genuinely parsed non-FFF admission witness exist?

**None is established by the retained audit evidence.** The sole SLA machine
request was exported as FFF. No successful non-FFF CLI admission, non-FFF
export or SLA slicing witness is claimed.

Source exposes alternate parsing channels: a typed SLA CLI option or imported
3MF config can make the local resolved technology non-FFF. However, absent an
observed experiment these are proposed **parsed-non-FFF rejection probes**,
not observed admitted cases. On the active `:3627-3633` path an actually
resolved SLA should be rejected, not admitted. The enum and validate's empty
error map cannot substitute for that missing runtime witness.

A future claim that non-FFF is admitted must retain the exact command/input,
producer identity, native status, effective typed/export value and the stage
that accepted it. If all genuinely parsed non-FFF inputs reject, record that
bounded admission domain as FFF-only with separately tested rejection; do not
invent a non-FFF admission fixture or relabel FFF export as one.

## Correct bounded domain and Rust destination

This is a Native **application/admission** test slice around the cited CLI
source boundary, not an Ares-owned pipeline feature or SLA port.

Destination:
`crates/ares-cli/tests/orca_parity/option_coverage/printer_technology.rs`,
existing `runner/application.rs` verification, and existing matrix
`domains/sweep/summary` accounting only where required to record the bounded
application scope. Core, WASM and rendering-neutral crates stay unchanged.
Use existing exact requested/effective checks and existing strict comparator.
No technology-specific SLA→FFF acceptance exception in `verify_value`.

The admitted domain is **FFF-only**. Its finite verification scope is
request channel × requested technology × resolved/effective value × observed
stage: a FFF admission control, immutable historical SLA-machine mismatch,
and a genuinely parsed SLA rejection parity witness. It is not `{FFF,SLA}`
passed as two admitted technologies through a generic motion sweep:

| Case | Required observation / bounded assertion | Current evidence |
|---|---|---|
| Explicit FFF machine + FFF companions | FFF requested/effective equality; admitted FFF export and existing strict API/CLI FFF output control | Future targeted receipt required |
| Original SLA machine JSON recipe | Successful Export but effective FFF; exact equality checker must report ApplicationMismatch | Observed both modes; original FAIL retained |
| Genuinely parsed SLA (typed CLI option or imported 3MF) | Native `:3627-3633` CLI_INVALID_PRINTER_TECH rejection; Ares CLI same input must match Native exit code and diagnostic | Proposed, unrun; rejection parity not SLA admission |

Do not advertise exhaustive combinations of presets, import formats,
printers, enum values or arbitrary malformed strings. The original generic
matrix failure remains discoverable in historical raw results even if a
future specialized expected-mismatch regression passes. A new row's PASS
means its explicitly named assertion passed, never that SLA application or
SLA slicing passed. Matrix reclassification requires separate review and
must link the original FAIL and scope/change rationale, not erase it.

## Included / deferred

Included: source/receipt mapping, exact requested/effective evidence,
FFF positive control, immutable original SLA mismatch, and **one** genuinely
parsed SLA rejection witness. Choose the typed CLI or imported 3MF channel
that both public CLIs can receive without translating SLA to FFF. Retain the
exact technology-bearing input bytes/token and each launch command.
Assert Native reaches the technology rejection (not an earlier unknown-option,
invalid-preset or schema error); assert Ares CLI on the same input has the
same exit code and diagnostic. If this parity assertion fails it is a public
RED, not an expected-Ares-failure PASS. No diagnostic weakening or
exit-code remapping is permitted. If platforms expose different process
status representations, document the observation;
do not silently treat distinct values as equivalent.

Use retained evidence before adding duplicate tests. Until the parsed SLA
rejection witness and matching Ares rejection are proven, this dimension
stays incomplete; successful FFF output alone cannot complete the scope.

Deferred: SLA slicing/geometry/resin config, SLAFullPrintConfig port,
production CLI parser repair, changing machine/process move semantics,
absence/default and conflicting-technology cases, technology precedence
redesign, broad inventory exclusions and universal
technology support. Any production repair needs its own public RED.
No normalization of SLA to FFF in the requested-value checker, no silent skip,
no fallback, no weakened comparator and no core filesystem/process behavior.

## Test-first / verification plan

1. Reuse/pin the original both-mode requested JSON/export/receipt hashes.
   A focused application test must first demonstrate the existing exact
   `verify_value(...,"printer_technology","SLA")` error with effective FFF.
   Preserve historical FAIL separately from the expected-mismatch control.
2. Produce the explicit FFF control with full channel attribution and exact
   requested/effective equality. Retain unchanged strict Native vs API/CLI
   complete-stream comparisons and raw API/CLI equality for admitted output.
3. Produce **one** typed CLI or imported-3MF SLA witness in official and seeded
   modes. Native must genuinely parse SLA and reach `:3627-3633` rejection
   (`CLI_INVALID_PRINTER_TECH`). Save input/command, typed/effective evidence,
   process exit code, diagnostic and stage. Run Ares CLI on the same
   technology-bearing input; persist stdout/stderr/status and require exact
   matching exit code and technology-rejection diagnostic. Ares acceptance,
   wrong exit code or different diagnostic is RED. No SLA slicing is
   in scope, and a JSON field alone does not prove actual parsing.
4. Stop if non-FFF is genuinely admitted contrary to the active
   source branch, or if producer/source identity differs. Report no admitted
   witness when rejection is the observed boundary; retain incomplete
   dimensions rather than manufacture coverage.
5. Only then propose specialized matrix accounting integration. Show before/
   after status alongside immutable original SLA FAIL. Existing byte
   comparator and `ApplicationMismatch` semantics remain unchanged.

Artifacts include command/producer hashes, input JSON and 3MF, requested/
effective ledger, status/stage receipts, exact original mismatch text and
all produced full streams.
